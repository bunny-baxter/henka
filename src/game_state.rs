use cgmath::{InnerSpace, Point3, point3, Vector2, vec2, Vector3, vec3};
use rand::Rng;
use winit::keyboard::KeyCode;

use crate::camera::Camera;
use crate::ecosim::{EcosimEntity, ecosim_tick};
use crate::fixed_point::Fixed;
use crate::render_util::Vertex;
use crate::physics_world::{PhysicsBody, PhysicsConfig, physics_tick};
use crate::voxel::{CHUNK_SIZE, VoxelChunk, VOXEL_SCALE, VOXEL_SIZE};
use crate::window::InputState;

const PHYSICS_SECONDS_PER_TICK: f64 = 1.0 / 60.0;

const ECOSIM_SECONDS_PER_TICK: f64 = 1.0 / 4.0;

const SUN_DISTANCE: f32 = 100.0;

// Number of shadow rays cast per voxel face. Each ray aims at a different
// random point across the sun's disk; the fraction that reach the sun becomes
// the light intensity, which softens shadow edges into a penumbra.
const SUN_LIGHT_SAMPLES: usize = 6;

// Radius of the sun's disk (in world units, at SUN_DISTANCE) that shadow rays
// are spread across. Larger values widen the penumbra / soften the shadows.
const SUN_SAMPLE_RADIUS: f32 = 16.0;

struct FirstPersonCameraController {
    pitch: f32,
    yaw: f32,
}

impl FirstPersonCameraController {
    fn get_forward(&self) -> Vector3<f32> {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos()
        )
    }

    fn get_camera_target(&self, position: &Point3<f32>) -> Point3<f32> {
        position + self.get_forward()
    }
}

struct OrbitCameraController {
    t: i32,
    height: f32,
    zoom: f32,
}

impl OrbitCameraController {
    fn get_camera_position(&self, target: &Point3<f32>) -> Point3<f32> {
        let orbit_t = self.t as f32 * 0.02;
        point3(
            target.x + orbit_t.cos() * self.zoom,
            target.y + self.height,
            target.z + orbit_t.sin() * self.zoom,
        )
    }
}

fn physics_point_to_world(physics_point: Point3<Fixed>) -> Point3<f32> {
    Fixed::point3_to_f32(physics_point) * VOXEL_SCALE
}

pub struct PlayerActor {
    pub body: PhysicsBody,
}

impl PlayerActor {
    fn new() -> Self {
        PlayerActor {
            body: PhysicsBody::new(),
        }
    }

    fn get_center_f32(&self) -> Point3<f32> {
        let pos = Fixed::point3_to_f32(self.body.position);
        let half_size = Fixed::vector3_to_f32(self.body.collision_size) * 0.5;
        point3(pos.x + half_size.x, pos.y + half_size.y, pos.z + half_size.z) * VOXEL_SCALE
    }

    fn get_center_base_f32(&self) -> Point3<f32> {
        let pos = Fixed::point3_to_f32(self.body.position);
        let half_size = Fixed::vector3_to_f32(self.body.collision_size) * 0.5;
        point3(pos.x + half_size.x, pos.y, pos.z + half_size.z) * VOXEL_SCALE
    }
}

fn point3_to_array(p: Point3<f32>) -> [f32; 3] {
    [p.x, p.y, p.z]
}

fn vector3_to_array(v: Vector3<f32>) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn calc_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let edge1 = vec3(p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]);
    let edge2 = vec3(p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]);
    let normal = edge1.cross(edge2).normalize();
    [normal.x, normal.y, normal.z]
}

// Function written by Claude, cleaned up by me
fn create_pyramid_mesh(offset: Point3<f32>, base_size: f32, height: f32, color: [f32; 3]) -> Vec<Vertex> {
    let half_base = base_size / 2.0;

    // Define the 5 vertex positions
    let apex_pos = [offset.x, offset.y + height, offset.z];
    let base_v0_pos = [offset.x - half_base, offset.y, offset.z - half_base];
    let base_v1_pos = [offset.x + half_base, offset.y, offset.z - half_base];
    let base_v2_pos = [offset.x + half_base, offset.y, offset.z + half_base];
    let base_v3_pos = [offset.x - half_base, offset.y, offset.z + half_base];

    // Create triangles for the pyramid with correct face normals
    // 4 triangular faces + 2 triangles for the square base = 18 vertices total

    // Front face
    let front_normal = calc_normal(base_v0_pos, apex_pos, base_v1_pos);
    let front_v0 = Vertex { position: base_v0_pos, light: color, uv: [0.0, 0.0], normal: front_normal };
    let front_apex = Vertex { position: apex_pos, light: color, uv: [0.5, 0.5], normal: front_normal };
    let front_v1 = Vertex { position: base_v1_pos, light: color, uv: [1.0, 0.0], normal: front_normal };

    // Right face
    let right_normal = calc_normal(base_v1_pos, apex_pos, base_v2_pos);
    let right_v1 = Vertex { position: base_v1_pos, light: color, uv: [0.0, 0.0], normal: right_normal };
    let right_apex = Vertex { position: apex_pos, light: color, uv: [0.5, 0.5], normal: right_normal };
    let right_v2 = Vertex { position: base_v2_pos, light: color, uv: [1.0, 0.0], normal: right_normal };

    // Back face
    let back_normal = calc_normal(base_v2_pos, apex_pos, base_v3_pos);
    let back_v2 = Vertex { position: base_v2_pos, light: color, uv: [0.0, 0.0], normal: back_normal };
    let back_apex = Vertex { position: apex_pos, light: color, uv: [0.5, 0.5], normal: back_normal };
    let back_v3 = Vertex { position: base_v3_pos, light: color, uv: [1.0, 0.0], normal: back_normal };

    // Left face
    let left_normal = calc_normal(base_v3_pos, apex_pos, base_v0_pos);
    let left_v3 = Vertex { position: base_v3_pos, light: color, uv: [0.0, 0.0], normal: left_normal };
    let left_apex = Vertex { position: apex_pos, light: color, uv: [0.5, 0.5], normal: left_normal };
    let left_v0 = Vertex { position: base_v0_pos, light: color, uv: [1.0, 0.0], normal: left_normal };

    // Base (two triangles) - normal points downward
    let base_normal = calc_normal(base_v0_pos, base_v1_pos, base_v2_pos);
    let base1_v0 = Vertex { position: base_v0_pos, light: color, uv: [0.0, 0.0], normal: base_normal };
    let base1_v1 = Vertex { position: base_v1_pos, light: color, uv: [1.0, 0.0], normal: base_normal };
    let base1_v2 = Vertex { position: base_v2_pos, light: color, uv: [1.0, 1.0], normal: base_normal };
    let base2_v0 = Vertex { position: base_v0_pos, light: color, uv: [0.0, 0.0], normal: base_normal };
    let base2_v2 = Vertex { position: base_v2_pos, light: color, uv: [1.0, 1.0], normal: base_normal };
    let base2_v3 = Vertex { position: base_v3_pos, light: color, uv: [0.0, 1.0], normal: base_normal };

    vec![
        // Front face
        front_v0, front_apex, front_v1,
        // Right face
        right_v1, right_apex, right_v2,
        // Back face
        back_v2, back_apex, back_v3,
        // Left face
        left_v3, left_apex, left_v0,
        // Base (two triangles)
        base1_v0, base1_v1, base1_v2,
        base2_v0, base2_v2, base2_v3,
    ]
}

fn get_entity_vertices(entity: &EcosimEntity, camera_pos: Point3<f32>) -> Vec<Vertex> {
    const QUAD_SIZE: f32 = 0.65;
    let pos = physics_point_to_world(entity.position);

    // Calculate UV offsets for sprite atlas (2x2 grid)
    let sprite_index = entity.flower_get_sprite_index();
    const FLOWER_UV_SCALE: f32 = 1.0 / 5.0;
    let uv_scale = FLOWER_UV_SCALE;
    let uv_offset_x = sprite_index.0 as f32 * uv_scale;
    let uv_offset_y = sprite_index.1 as f32 * uv_scale;

    // Billboard: calculate direction from flower to camera (only in XZ plane)
    let to_camera = camera_pos - pos;
    let to_camera_xz = vec3(to_camera.x, 0.0, to_camera.z).normalize();

    // Right vector perpendicular to camera direction
    let right = vec3(-to_camera_xz.z, 0.0, to_camera_xz.x) * (QUAD_SIZE / 2.0);

    // Create a single quad facing the camera
    let base_left_pos = point3_to_array(pos - right);
    let base_right_pos = point3_to_array(pos + right);
    let top_left_pos = point3_to_array(pos - right + vec3(0.0, QUAD_SIZE, 0.0));
    let top_right_pos = point3_to_array(pos + right + vec3(0.0, QUAD_SIZE, 0.0));

    let normal = calc_normal(base_left_pos, base_right_pos, top_left_pos);

    let base_left = Vertex { position: base_left_pos, light: [0.0, 0.0, 0.0], uv: [uv_offset_x, uv_offset_y + uv_scale], normal };
    let base_right = Vertex { position: base_right_pos, light: [0.0, 0.0, 0.0], uv: [uv_offset_x + uv_scale, uv_offset_y + uv_scale], normal };
    let top_left = Vertex { position: top_left_pos, light: [0.0, 0.0, 0.0], uv: [uv_offset_x, uv_offset_y], normal };
    let top_right = Vertex { position: top_right_pos, light: [0.0, 0.0, 0.0], uv: [uv_offset_x + uv_scale, uv_offset_y], normal };

    vec![
        base_left, top_left, top_right,
        top_right, base_right, base_left,
    ]
}

pub struct GameState {
    pub exit: bool,
    pub window_size: Vector2<u32>,
    pub chunk: VoxelChunk,
    pub camera: Camera,
    first_person_camera_controller: FirstPersonCameraController,
    orbit_camera_controller: OrbitCameraController,
    is_camera_first_person: bool,
    physics_tick_accumulator: f64,
    physics_config: PhysicsConfig,
    pub player: PlayerActor,
    ecosim_tick_accumulator: f64,
    pub ecosim_entities: Vec<EcosimEntity>,
    pub sun_time: f64,
    pub sun_position: Vector3<f32>,
    // Fixed random offsets (in the sun's disk plane) used to aim shadow rays at
    // several points across the sun. Chosen once so the soft shadows change
    // smoothly as the sun moves rather than flickering frame to frame.
    sun_sample_offsets: Vec<Vector2<f32>>,
}

impl GameState {
    pub fn new() -> Self {
        let mut player = PlayerActor::new();
        player.body.position = point3(Fixed::new(2, 0), Fixed::new(3, 0), Fixed::new(2, 0));
        player.body.collision_size = vec3(Fixed::new(0, 128), Fixed::new(2, 0), Fixed::new(0, 128));
        // Distribute the shadow-ray sample points uniformly across the sun's disk.
        let mut rng = rand::rng();
        let sun_sample_offsets = (0..SUN_LIGHT_SAMPLES).map(|_| {
            let radius = SUN_SAMPLE_RADIUS * rng.random::<f32>().sqrt();
            let angle = std::f32::consts::TAU * rng.random::<f32>();
            vec2(radius * angle.cos(), radius * angle.sin())
        }).collect();
        GameState {
            exit: false,
            window_size: vec2(0, 0),
            chunk: VoxelChunk::new(),
            camera: Camera::new(point3(-2.0, 0.0, 2.0), point3(0.25, 0.25, 0.25), 0.0),
            first_person_camera_controller: FirstPersonCameraController {
                pitch: 0.0,
                yaw: 0.0,
            },
            orbit_camera_controller: OrbitCameraController {
                t: 0,
                zoom: 1.4,
                height: 0.6,
            },
            is_camera_first_person: true,
            physics_tick_accumulator: 0.0,
            physics_config: PhysicsConfig { gravity: vec3(Fixed::ZERO, -Fixed::new(0, 3), Fixed::ZERO) },
            player,
            ecosim_tick_accumulator: 0.0,
            ecosim_entities: vec![],
            sun_time: 0.0,
            sun_position: vec3(0.0, 1.0, 0.0),
            sun_sample_offsets,
        }
    }

    pub fn set_window_size(&mut self, window_size: Vector2<u32>) {
        self.window_size = window_size;
        self.camera.aspect_ratio = window_size.x as f32 / window_size.y as f32;
    }

    pub fn generate_voxels(&mut self) {
        for i in 2..CHUNK_SIZE.x {
            for k in 2..CHUNK_SIZE.z {
                for j in 0..3 {
                    self.chunk.set_voxel(vec3(i, j, k), 1);
                }
            }
        }
        self.chunk.set_voxel(vec3(4, 4, 4), 1);

        self.chunk.set_voxel(vec3(3, 3, 2), 1);
        self.chunk.set_voxel(vec3(4, 3, 2), 1);
        self.chunk.set_voxel(vec3(4, 4, 2), 1);

        self.chunk.set_voxel(vec3(12, 3, 12), 1);
        self.chunk.set_voxel(vec3(13, 3, 12), 1);
        self.chunk.set_voxel(vec3(14, 3, 12), 1);
        self.chunk.set_voxel(vec3(12, 3, 13), 1);
        self.chunk.set_voxel(vec3(13, 3, 13), 1);
        self.chunk.set_voxel(vec3(14, 3, 13), 1);
        self.chunk.set_voxel(vec3(12, 3, 14), 1);
        self.chunk.set_voxel(vec3(13, 3, 14), 1);
        self.chunk.set_voxel(vec3(14, 3, 14), 1);
        self.chunk.set_voxel(vec3(13, 4, 13), 1);

        self.ecosim_entities.push(EcosimEntity::new(vec3(8, 3, 3)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(9, 3, 4)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(12, 3, 8)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(12, 3, 8)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(24, 3, 8)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(14, 3, 30)));
        self.ecosim_entities.push(EcosimEntity::new(vec3(6, 3, 2)));
        for e in self.ecosim_entities.iter_mut() {
            e.randomize_genome();
        }
    }

    pub fn on_key_pressed(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::KeyQ => self.exit = true,
            KeyCode::KeyC => self.is_camera_first_person = !self.is_camera_first_person,
            KeyCode::Space => if self.player.body.is_on_ground {
                self.player.body.velocity.y = Fixed::new(0, 48);
            },
            _ => (),
        };
    }

    // Casts several rays from a voxel face toward random points across the sun's
    // disk. Returns the fraction that reach the sun unobstructed, so faces on the
    // shadow boundary come out partially lit (a soft penumbra) rather than fully
    // lit or fully dark. `sample_directions` are the per-sample ray directions
    // shared by every face this frame (see calculate_light).
    fn light_raycast(&self, voxel_coord: Vector3<usize>, face_normal: Vector3<f32>, sample_directions: &[Vector3<f32>]) -> f32 {
        // Start the ray just outside the face so it doesn't immediately hit
        // the originating voxel or its solid neighbors below the face plane.
        const FACE_OFFSET: f32 = 0.001;
        let voxel_center = vec3(
            voxel_coord.x as f32 * VOXEL_SIZE.x + VOXEL_SIZE.x / 2.0,
            voxel_coord.y as f32 * VOXEL_SIZE.y + VOXEL_SIZE.y / 2.0,
            voxel_coord.z as f32 * VOXEL_SIZE.z + VOXEL_SIZE.z / 2.0);
        let face_origin = voxel_center + face_normal * (VOXEL_SCALE / 2.0 + FACE_OFFSET);
        let lit_samples = sample_directions.iter()
            .filter(|direction| self.chunk.raycast(face_origin, **direction).is_none())
            .count();
        lit_samples as f32 / sample_directions.len() as f32
    }

    fn calculate_light(&mut self) {
        const FACE_NORMALS: [Vector3<f32>; 6] = [
            vec3(1.0, 0.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            vec3(0.0, -1.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, -1.0),
        ];
        // Build this frame's shadow-ray directions by spreading the fixed sample
        // offsets across the sun's disk. The disk plane is rebuilt from the
        // current sun direction each frame, so the sampling follows the sun and
        // the soft shadows shift smoothly instead of flickering.
        //
        // The sun orbits in the y-z plane, so the world x-axis is always
        // perpendicular to the sun direction. Using it as the cross reference
        // keeps the disk basis continuous across the whole orbit (a world-up
        // reference would spin unstably as the sun passes overhead/underfoot).
        let sun_direction = (-self.sun_position).normalize();
        let right = sun_direction.cross(vec3(1.0, 0.0, 0.0)).normalize();
        let up = sun_direction.cross(right).normalize();
        let sample_directions: Vec<Vector3<f32>> = self.sun_sample_offsets.iter()
            .map(|offset| sun_direction * SUN_DISTANCE + right * offset.x + up * offset.y)
            .collect();
        for i in 0..CHUNK_SIZE.x {
            for j in 0..CHUNK_SIZE.y {
                for k in 0..CHUNK_SIZE.z {
                    let coord = vec3(i, j, k);
                    if self.chunk.get_voxel(coord) == 0 {
                        continue;
                    }
                    let coord_i32 = vec3(i as i32, j as i32, k as i32);
                    let k_t = k as f32 / CHUNK_SIZE.z as f32;
                    for face_normal in FACE_NORMALS.iter() {
                        let face_dir_i32 = vec3(face_normal.x as i32, face_normal.y as i32, face_normal.z as i32);
                        if !self.chunk.is_face_visible(coord_i32, face_dir_i32) {
                            continue;
                        }
                        // A face angled away from the sun can never see it: a
                        // shadow ray toward the sun would just curve back into
                        // this voxel. Skip the raycasts and mark it fully shadowed.
                        let light_intensity = if face_normal.dot(sun_direction) < 0.0 {
                            0.0
                        } else {
                            self.light_raycast(coord, *face_normal, &sample_directions)
                        };
                        let light = [k_t * light_intensity, k_t * light_intensity, 1.0 * light_intensity];
                        self.chunk.set_voxel_face_light(coord, [face_normal.x, face_normal.y, face_normal.z], light);
                    }
                }
            }
        }
    }

    pub fn update(&mut self, dt: f64, input_state: &InputState) {
        self.physics_tick_accumulator += dt;
        while self.physics_tick_accumulator > PHYSICS_SECONDS_PER_TICK {
            const PLAYER_SPEED: f32 = 0.05;
            let forward = self.first_person_camera_controller.get_forward();
            let forward_xz = vec3(forward.x, 0.0, forward.z).normalize();
            let forward_velocity = Fixed::vector3_from_f32(forward_xz * PLAYER_SPEED);
            let right = forward_xz.cross(cgmath::Vector3::unit_y()).normalize();
            let right_velocity = Fixed::vector3_from_f32(right * PLAYER_SPEED);
            self.player.body.velocity.x = Fixed::ZERO;
            self.player.body.velocity.z = Fixed::ZERO;
            if input_state.is_key_pressed(KeyCode::KeyW) {
                self.player.body.velocity.x += forward_velocity.x;
                self.player.body.velocity.z += forward_velocity.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyS) {
                self.player.body.velocity.x -= forward_velocity.x;
                self.player.body.velocity.z -= forward_velocity.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyD) {
                self.player.body.velocity.x += right_velocity.x;
                self.player.body.velocity.z += right_velocity.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyA) {
                self.player.body.velocity.x -= right_velocity.x;
                self.player.body.velocity.z -= right_velocity.z;
            }
            physics_tick(&self.physics_config, std::slice::from_mut(&mut self.player.body), &self.chunk);
            self.physics_tick_accumulator -= PHYSICS_SECONDS_PER_TICK;
        }

        self.ecosim_tick_accumulator += dt;
        while self.ecosim_tick_accumulator > ECOSIM_SECONDS_PER_TICK {
            ecosim_tick(&mut self.ecosim_entities, &self.chunk);
            self.ecosim_tick_accumulator -= ECOSIM_SECONDS_PER_TICK;
        }

        if self.is_camera_first_person {
            // Mouse control (primary input)
            const MOUSE_SENSITIVITY: f32 = 0.003;
            self.first_person_camera_controller.yaw += input_state.mouse_delta.x as f32 * MOUSE_SENSITIVITY;
            self.first_person_camera_controller.pitch -= input_state.mouse_delta.y as f32 * MOUSE_SENSITIVITY;

            // Arrow keys (alternative control)
            if input_state.is_key_pressed(KeyCode::ArrowUp) {
                self.first_person_camera_controller.pitch += 0.01;
            }
            if input_state.is_key_pressed(KeyCode::ArrowDown) {
                self.first_person_camera_controller.pitch -= 0.01;
            }
            if input_state.is_key_pressed(KeyCode::ArrowRight) {
                self.first_person_camera_controller.yaw += 0.01;
            }
            if input_state.is_key_pressed(KeyCode::ArrowLeft) {
                self.first_person_camera_controller.yaw -= 0.01;
            }

            let player_center_base = self.player.get_center_base_f32();
            let eye_height = self.player.body.collision_size.y.to_f32() * VOXEL_SCALE * 0.95;
            self.camera.position = player_center_base + vec3(0.0, eye_height, 0.0);
            self.camera.target = self.first_person_camera_controller.get_camera_target(&self.camera.position);
        } else {
            if input_state.is_key_pressed(KeyCode::ArrowUp) {
                self.orbit_camera_controller.zoom -= 0.01;
            }
            if input_state.is_key_pressed(KeyCode::ArrowDown) {
                self.orbit_camera_controller.zoom += 0.01;
            }
            if self.orbit_camera_controller.zoom < 0.5 {
                self.orbit_camera_controller.zoom = 0.5;
            }
            if input_state.is_key_pressed(KeyCode::ArrowRight) {
                self.orbit_camera_controller.t += 1;
            }
            if input_state.is_key_pressed(KeyCode::ArrowLeft) {
                self.orbit_camera_controller.t -= 1;
            }
            if input_state.is_key_pressed(KeyCode::KeyJ) {
                self.orbit_camera_controller.height -= 0.05;
            }
            if input_state.is_key_pressed(KeyCode::KeyK) {
                self.orbit_camera_controller.height += 0.05;
            }
            // Center the camera target on the player's hitbox
            self.camera.target = self.player.get_center_f32();
            self.camera.position = self.orbit_camera_controller.get_camera_position(&self.camera.target);
        }

        self.sun_time += 0.2 * dt;
        self.sun_position = vec3(0.0, -self.sun_time.sin() as f32, self.sun_time.cos() as f32);
        self.calculate_light();
    }

    pub fn get_voxel_vertices(&mut self) -> Vec<Vertex> {
        let mut vertices = self.chunk.get_vertices();
        // Center the player model on the hitbox base
        vertices.append(&mut create_pyramid_mesh(
                self.player.get_center_base_f32(),
                self.player.body.collision_size.x.to_f32() * VOXEL_SCALE,
                self.player.body.collision_size.y.to_f32() * VOXEL_SCALE,
                [1.0, 1.0, 0.0]));
        vertices.append(&mut self.get_raycast_debug_vertices());
        vertices
    }

    fn get_raycast_debug_vertices(&self) -> Vec<Vertex> {
        let ray_origin = vec3(self.camera.position.x, self.camera.position.y, self.camera.position.z);
        let ray_direction = (self.camera.target - self.camera.position).normalize();
        let Some(t) = self.chunk.raycast(ray_origin, ray_direction) else {
            return vec![];
        };
        let hit = self.camera.position + ray_direction * t;
        create_pyramid_mesh(hit, 0.15, 0.15, [1.0, 0.0, 1.0])
    }

    pub fn get_sun_vertices(&self) -> Vec<Vertex> {
        const SUN_QUAD_SIZE: f32 = 25.0;

        let sun_direction = (-self.sun_position).normalize();
        let center = self.camera.position + sun_direction * SUN_DISTANCE;

        let world_up = vec3(0.0, 1.0, 0.0);
        let right = world_up.cross(sun_direction).normalize() * (SUN_QUAD_SIZE / 2.0);
        let up = sun_direction.cross(right.normalize()).normalize() * (SUN_QUAD_SIZE / 2.0);

        // Always upright version
        //let forward = (self.camera.target - self.camera.position).normalize();
        //let right = forward.cross(vec3(0.0, 1.0, 0.0)).normalize() * (SUN_QUAD_SIZE / 2.0);
        //let up = forward.cross(right.normalize()).normalize() * (SUN_QUAD_SIZE / 2.0);

        let bl = point3_to_array(center - right - up);
        let br = point3_to_array(center + right - up);
        let tl = point3_to_array(center - right + up);
        let tr = point3_to_array(center + right + up);

        let normal = vector3_to_array(sun_direction);

        vec![
            Vertex { position: bl, light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal },
            Vertex { position: tl, light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal },
            Vertex { position: tr, light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal },
            Vertex { position: tr, light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal },
            Vertex { position: br, light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal },
            Vertex { position: bl, light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal },
        ]
    }

    pub fn get_flower_vertices(&self) -> Vec<Vertex> {
        let mut result = vec![];
        // Sort entities by distance to camera because depth buffer writing is disabled
        let camera_pos = self.camera.position;
        let mut entities_with_distance: Vec<(&EcosimEntity, f32)> = self.ecosim_entities.iter()
            .map(|e| {
                let entity_pos = physics_point_to_world(e.position);
                let distance = (camera_pos - entity_pos).magnitude();
                (e, distance)
            })
            .collect();
        entities_with_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // Sort descending
        for (entity, _) in entities_with_distance {
            result.append(&mut get_entity_vertices(entity, camera_pos));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Runs the lighting pass over a flat ground with a thin raised wall that
    // casts a shadow, and returns the lit fraction of every ground top face.
    // The blue channel of the stored light equals the raw lit fraction (see
    // calculate_light), which is what soft shadows should turn fractional.
    fn ground_top_face_intensities(sun_position: Vector3<f32>) -> Vec<f32> {
        let mut state = GameState::new();
        for x in 0..CHUNK_SIZE.x {
            for z in 0..CHUNK_SIZE.z {
                state.chunk.set_voxel(vec3(x, 0, z), 1);
            }
        }
        // A wall running along x, so it casts a shadow that moves in z.
        for x in 4..28 {
            for y in 1..10 {
                state.chunk.set_voxel(vec3(x, y, 12), 1);
            }
        }
        // Build the geometry first so the lighting pass has vertices to write to.
        let _ = state.chunk.get_vertices();
        state.sun_position = sun_position;
        state.calculate_light();

        state.chunk.get_vertices().iter()
            .filter(|v| v.normal == [0.0, 1.0, 0.0] && (v.position[1] - VOXEL_SIZE.y).abs() < 0.01)
            .map(|v| v.light[2])
            .collect()
    }

    // Shadows should have a soft penumbra: at the shadow boundary, some ground
    // faces are only partially lit rather than every face being fully lit or
    // fully dark (which is all the old single-ray-to-sun-center test produced).
    #[test]
    fn soft_shadows_have_a_penumbra() {
        // Sun above the horizon, tilted along z so the wall casts a shadow.
        let sun_position = vec3(0.0, -(0.6f32).sin(), (0.6f32).cos());
        let intensities = ground_top_face_intensities(sun_position);

        let fully_lit = intensities.iter().filter(|&&i| i > 0.99).count();
        let fully_dark = intensities.iter().filter(|&&i| i < 0.01).count();
        let penumbra = intensities.iter().filter(|&&i| i > 0.01 && i < 0.99).count();

        assert!(fully_lit > 0, "expected some ground in full sunlight");
        assert!(fully_dark > 0, "expected some ground in full shadow");
        assert!(penumbra > 0, "expected a soft penumbra of partially-lit faces, got none");
    }

    // As the sun moves a small amount, a boundary face's lit fraction should
    // change smoothly (through intermediate values) rather than snapping
    // straight from lit to dark.
    #[test]
    fn shadow_edge_changes_smoothly() {
        let mut saw_intermediate = false;
        for step in 0..40 {
            let t = 0.55 + step as f32 * 0.002;
            let sun_position = vec3(0.0, -t.sin(), t.cos());
            let intensities = ground_top_face_intensities(sun_position);
            if intensities.iter().any(|&i| i > 0.2 && i < 0.8) {
                saw_intermediate = true;
                break;
            }
        }
        assert!(saw_intermediate, "shadow edge never passed through intermediate light values");
    }
}
