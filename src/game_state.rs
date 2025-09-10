use cgmath::{point3, Vector2, vec2, vec3};
use winit::keyboard::KeyCode;

use crate::voxel::VoxelChunk;
use crate::camera::Camera;

struct OrbitCameraController {
    t: i32,
    zoom: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

pub struct GameState {
    pub exit: bool,
    pub window_size: Vector2<u32>,
    pub chunk: VoxelChunk,
    pub camera: Camera,
    camera_controller: OrbitCameraController
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            exit: false,
            window_size: vec2(0, 0),
            chunk: VoxelChunk::new(),
            camera: Camera::new(point3(-2.0, 0.0, 2.0), point3(0.25, 0.25, 0.25), 0.0),
            camera_controller: OrbitCameraController {
                t: 0,
                zoom: 1.4,
                is_forward_pressed: false,
                is_backward_pressed: false,
                is_left_pressed: false,
                is_right_pressed: false,
            },
        }
    }

    pub fn set_window_size(&mut self, window_size: Vector2<u32>) {
        self.window_size = window_size;
        self.camera.aspect_ratio = window_size.x as f32 / window_size.y as f32;
    }

    pub fn generate_voxels(&mut self) {
        self.chunk.set_voxel(vec3(0, 0, 0), 1);
        self.chunk.set_voxel(vec3(1, 0, 0), 1);
        self.chunk.set_voxel(vec3(0, 0, 1), 1);
        self.chunk.set_voxel(vec3(1, 0, 1), 1);
        self.chunk.set_voxel(vec3(0, 1, 0), 1);
    }

    pub fn on_key_pressed(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::KeyQ => self.exit = true,
            KeyCode::KeyW | KeyCode::ArrowUp => self.camera_controller.is_forward_pressed = true,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.camera_controller.is_left_pressed = true,
            KeyCode::KeyS | KeyCode::ArrowDown => self.camera_controller.is_backward_pressed = true,
            KeyCode::KeyD | KeyCode::ArrowRight => self.camera_controller.is_right_pressed = true,
            _ => (),
        };
    }

    pub fn on_key_released(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::KeyW | KeyCode::ArrowUp => self.camera_controller.is_forward_pressed = false,
            KeyCode::KeyA | KeyCode::ArrowLeft => self.camera_controller.is_left_pressed = false,
            KeyCode::KeyS | KeyCode::ArrowDown => self.camera_controller.is_backward_pressed = false,
            KeyCode::KeyD | KeyCode::ArrowRight => self.camera_controller.is_right_pressed = false,
            _ => (),
        };
    }

    pub fn update(&mut self) {
        if self.camera_controller.is_forward_pressed {
            self.camera_controller.zoom -= 0.01;
        }
        if self.camera_controller.is_backward_pressed {
            self.camera_controller.zoom += 0.01;
        }
        if self.camera_controller.zoom < 0.5 {
            self.camera_controller.zoom = 0.5;
        }
        if self.camera_controller.is_right_pressed {
            self.camera_controller.t += 1;
        }
        if self.camera_controller.is_left_pressed {
            self.camera_controller.t -= 1;
        }
        let orbit_t = self.camera_controller.t as f32 * 0.02;
        self.camera.position = point3(
            self.camera.target.x + orbit_t.cos() * self.camera_controller.zoom,
            self.camera.target.y + 0.6,
            self.camera.target.z + orbit_t.sin() * self.camera_controller.zoom,
        );
    }
}
