use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use cgmath::{vec2, Vector2};
use pollster::FutureExt as _;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
use wgpu::util::DeviceExt;
use wgpu_text::{glyph_brush::{Section as TextSection, OwnedText, ab_glyph::FontRef, OwnedSection}, BrushBuilder, TextBrush};

use crate::camera::CameraUniform;
use crate::game_state::GameState;
use crate::render_util::MovingAverage;
use crate::texture::DepthTexture;
use crate::voxel::Vertex;

struct TimestampQueryState {
    query_set: wgpu::QuerySet,
    buffer: wgpu::Buffer,
    readback_buffer: Arc<wgpu::Buffer>,
    last_render_measurement_ns: Arc<AtomicU64>,
}

struct RenderState<'a> {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    timestamp_query_state: Option<TimestampQueryState>,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: DepthTexture,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    #[allow(unused)]
    font: &'a [u8],
    text_brush: TextBrush<FontRef<'a>>,
    text_section: OwnedSection,
}

impl RenderState<'_> {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let window_arc = Arc::new(window);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window_arc.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let timestamp_query_enabled = !(adapter.features() & wgpu::Features::TIMESTAMP_QUERY).is_empty();
        let required_features = if timestamp_query_enabled {
            wgpu::Features::TIMESTAMP_QUERY
        } else {
            wgpu::Features::empty()
        };

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features,
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            },
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let depth_texture = DepthTexture::new(&device, &config, "depth_texture");

        let camera_uniform = CameraUniform::new();
        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let depth_stencil_state = wgpu::DepthStencilState {
            format: DepthTexture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        };

        let font = include_bytes!("Rubik-Regular.ttf");
        let text_brush = BrushBuilder::using_font_bytes(font).unwrap()
            .with_depth_stencil(Some(depth_stencil_state.clone()))
            .build(&device, config.width, config.height, config.format);
        let text_section = TextSection::default()
            .with_bounds((config.width as f32 * 0.4, config.height as f32))
            .with_screen_position((32.0, 32.0))
            .to_owned();

        let timestamp_query_state = if timestamp_query_enabled {
            let timestamp_query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp Query Set"),
                ty: wgpu::QueryType::Timestamp,
                count: 2, // start and end
            });
            let timestamp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timestamp Buffer"),
                size: 16, // 2 timestamps * 8 bytes each
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let timestamp_readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Timestamp Readback"),
                size: 16,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            Some(TimestampQueryState {
                query_set: timestamp_query_set,
                buffer: timestamp_buffer,
                readback_buffer: Arc::new(timestamp_readback_buffer),
                last_render_measurement_ns: Arc::new(0.into()),
            })
        } else {
            None
        };

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::buffer_layout()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(depth_stencil_state),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        RenderState {
            window: window_arc,
            surface,
            device,
            queue,
            config,
            timestamp_query_state,
            render_pipeline,
            depth_texture,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            font,
            text_brush,
            text_section,
        }
    }

    fn write_buffers(&mut self) {
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
        self.text_brush.queue(&self.device, &self.queue, [&self.text_section]).unwrap();
    }

    fn render(&mut self, vertices: &Vec<Vertex>) -> Result<(), wgpu::SurfaceError> {
        let vertex_buffer = self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: self.timestamp_query_state.as_ref().map(|qs| {
                    wgpu::RenderPassTimestampWrites {
                        query_set: &qs.query_set,
                        beginning_of_pass_write_index: Some(0),
                        end_of_pass_write_index: Some(1),
                    }
                }),
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

            let n_vertices = vertices.len() as u32;
            render_pass.draw(0..n_vertices, 0..1);

            self.text_brush.draw(&mut render_pass);
        }

        if let Some(qs) = self.timestamp_query_state.as_ref() {
            encoder.resolve_query_set(&qs.query_set, 0..2, &qs.buffer, 0);
            encoder.copy_buffer_to_buffer(&qs.buffer, 0, &qs.readback_buffer, 0, 16);

            // Ensure the previous frame's timestamp callback has run so the readback buffer is unmapped before we call `submit`.
            self.device.poll(wgpu::PollType::Wait).unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        if let Some(qs) = self.timestamp_query_state.as_ref() {
            let readback_ref = qs.readback_buffer.clone();
            let measurement_ref = qs.last_render_measurement_ns.clone();
            qs.readback_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    {
                        let data = readback_ref.slice(..).get_mapped_range();
                        let timestamps: &[u64] = bytemuck::cast_slice(&data);
                        let duration_ns = timestamps[1].saturating_sub(timestamps[0]);
                        if duration_ns > 0 {
                            measurement_ref.store(duration_ns, Ordering::Relaxed);
                        }
                    }
                    readback_ref.unmap();
                }
            });
        }

        Ok(())
    }
}

pub struct InputState {
    keys: HashSet<KeyCode>,
    left_mouse: bool,
}

impl InputState {
    fn new() -> Self {
        Self {
            keys: HashSet::new(),
            left_mouse: false,
        }
    }

    fn on_key_pressed(&mut self, key_code: KeyCode) {
        self.keys.insert(key_code);
    }

    fn on_key_released(&mut self, key_code: KeyCode) {
        self.keys.remove(&key_code);
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys.contains(&key_code)
    }

    pub fn is_left_mouse_pressed(&self) -> bool {
        self.left_mouse
    }
}

struct App<'a> {
    frame_count: u64,
    last_frame: Instant,

    render_state: Option<RenderState<'a>>,
    input_state: InputState,
    game_state: GameState,

    frame_delta: MovingAverage,
    update_time: MovingAverage,
}

impl<'a> App<'a> {
    fn new() -> Self {
        App {
            frame_count: 0,
            last_frame: Instant::now(),
            render_state: None,
            input_state: InputState::new(),
            game_state: GameState::new(),
            frame_delta: MovingAverage::new(100),
            update_time: MovingAverage::new(100),
        }
    }

    fn render_state(&self) -> &RenderState<'a> {
        self.render_state.as_ref().unwrap()
    }

    fn render_state_mut(&mut self) -> &mut RenderState<'a> {
        self.render_state.as_mut().unwrap()
    }

    fn get_window_size(&self) -> Vector2<u32> {
        let render_state = self.render_state();
        return vec2(render_state.config.width, render_state.config.height);
    }

    async fn init_render_state(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Henka")
            .with_inner_size(winit::dpi::PhysicalSize::new(1920, 1080));
        let window = event_loop.create_window(window_attributes).unwrap();
        self.render_state = Some(RenderState::new(window).await);

        self.game_state.set_window_size(self.get_window_size());
        self.game_state.generate_voxels();
    }

    fn update(&mut self) {
        self.game_state.update(&self.input_state);
        let view_projection = self.game_state.camera.build_view_projection_matrix();
        self.render_state_mut().camera_uniform.set_view_projection(view_projection);
        if self.frame_count % 20 == 0 {
            let fps_str = format!("{:.2} fps \n", 1.0 / self.frame_delta.get_average());
            let update_time_str = format!("update: {:.2}ms \n", self.update_time.get_average());
            let render_time_str = match self.render_state().timestamp_query_state.as_ref() {
                Some(qs) => {
                    let render_measurement_ns = qs.last_render_measurement_ns.load(Ordering::Relaxed);
                    let render_time_ms = (self.render_state().queue.get_timestamp_period() as f64 * render_measurement_ns as f64) / 1000000.0;
                    format!("render: {:.2}ms \n", render_time_ms)
                },
                None => "n/a".to_string(),
            };
            self.render_state_mut().text_section.text = vec![
                OwnedText::new(fps_str).with_scale(64.0).with_color([1.0, 1.0, 0.0, 1.0]),
                OwnedText::new(update_time_str).with_scale(64.0).with_color([1.0, 1.0, 0.0, 1.0]),
                OwnedText::new(render_time_str).with_scale(64.0).with_color([1.0, 1.0, 0.0, 1.0]),
            ];
        }
        self.render_state_mut().write_buffers();
        self.frame_count += 1;
    }

    fn render(&mut self) {
        let vertices = self.game_state.get_vertices();
        self.render_state_mut().render(&vertices).unwrap();
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_render_state(event_loop).block_on();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.game_state.exit = true,
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                if state == ElementState::Pressed {
                    self.input_state.on_key_pressed(key_code);
                    self.game_state.on_key_pressed(key_code);
                } else if state == ElementState::Released {
                    self.input_state.on_key_released(key_code);
                }
            },
            WindowEvent::RedrawRequested => {
                let frame_delta = self.last_frame.elapsed().as_secs_f64();
                self.frame_delta.add_sample(frame_delta);
                self.last_frame = Instant::now();

                let pre_update = Instant::now();
                self.update();
                self.update_time.add_sample(pre_update.elapsed().as_micros() as f64 / 1000.0);

                self.render();
                self.render_state().window.request_redraw();
            }
            _ => (),
        }
        if self.game_state.exit {
            event_loop.exit();
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
