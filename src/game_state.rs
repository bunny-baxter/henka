use cgmath::{InnerSpace, Point3, point3, Vector2, vec2, Vector3, vec3};
use winit::keyboard::KeyCode;

use crate::camera::Camera;
use crate::fixed_point::Fixed;
use crate::render_util::Vertex;
use crate::physics_world::{PhysicsBody, PhysicsConfig, physics_tick};
use crate::voxel::{CHUNK_SIZE, VoxelChunk, VOXEL_SIZE};
use crate::window::InputState;

const PHYSICS_SECONDS_PER_TICK: f64 = 1.0 / 60.0;

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

pub struct PlayerActor {
    pub body: PhysicsBody,
}

impl PlayerActor {
    fn new() -> Self {
        PlayerActor {
            body: PhysicsBody::new(),
        }
    }
}

// Function written by Claude, cleaned up by me
fn create_pyramid_mesh(offset: Point3<f32>, base_size: f32, height: f32) -> Vec<Vertex> {
    const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
    let half_base = base_size / 2.0;

    // Helper function to calculate face normal from three vertices
    let calc_normal = |p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]| -> [f32; 3] {
        let edge1 = vec3(p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]);
        let edge2 = vec3(p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]);
        let normal = edge1.cross(edge2).normalize();
        [normal.x, normal.y, normal.z]
    };

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
    let front_v0 = Vertex { position: base_v0_pos, color: YELLOW, uv: [0.0, 0.0], normal: front_normal };
    let front_apex = Vertex { position: apex_pos, color: YELLOW, uv: [0.5, 0.5], normal: front_normal };
    let front_v1 = Vertex { position: base_v1_pos, color: YELLOW, uv: [1.0, 0.0], normal: front_normal };

    // Right face
    let right_normal = calc_normal(base_v1_pos, apex_pos, base_v2_pos);
    let right_v1 = Vertex { position: base_v1_pos, color: YELLOW, uv: [0.0, 0.0], normal: right_normal };
    let right_apex = Vertex { position: apex_pos, color: YELLOW, uv: [0.5, 0.5], normal: right_normal };
    let right_v2 = Vertex { position: base_v2_pos, color: YELLOW, uv: [1.0, 0.0], normal: right_normal };

    // Back face
    let back_normal = calc_normal(base_v2_pos, apex_pos, base_v3_pos);
    let back_v2 = Vertex { position: base_v2_pos, color: YELLOW, uv: [0.0, 0.0], normal: back_normal };
    let back_apex = Vertex { position: apex_pos, color: YELLOW, uv: [0.5, 0.5], normal: back_normal };
    let back_v3 = Vertex { position: base_v3_pos, color: YELLOW, uv: [1.0, 0.0], normal: back_normal };

    // Left face
    let left_normal = calc_normal(base_v3_pos, apex_pos, base_v0_pos);
    let left_v3 = Vertex { position: base_v3_pos, color: YELLOW, uv: [0.0, 0.0], normal: left_normal };
    let left_apex = Vertex { position: apex_pos, color: YELLOW, uv: [0.5, 0.5], normal: left_normal };
    let left_v0 = Vertex { position: base_v0_pos, color: YELLOW, uv: [1.0, 0.0], normal: left_normal };

    // Base (two triangles) - normal points downward
    let base_normal = calc_normal(base_v0_pos, base_v1_pos, base_v2_pos);
    let base1_v0 = Vertex { position: base_v0_pos, color: YELLOW, uv: [0.0, 0.0], normal: base_normal };
    let base1_v1 = Vertex { position: base_v1_pos, color: YELLOW, uv: [1.0, 0.0], normal: base_normal };
    let base1_v2 = Vertex { position: base_v2_pos, color: YELLOW, uv: [1.0, 1.0], normal: base_normal };
    let base2_v0 = Vertex { position: base_v0_pos, color: YELLOW, uv: [0.0, 0.0], normal: base_normal };
    let base2_v2 = Vertex { position: base_v2_pos, color: YELLOW, uv: [1.0, 1.0], normal: base_normal };
    let base2_v3 = Vertex { position: base_v3_pos, color: YELLOW, uv: [0.0, 1.0], normal: base_normal };

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
}

impl GameState {
    pub fn new() -> Self {
        let mut player = PlayerActor::new();
        player.body.position = point3(Fixed::new(2, 0), Fixed::new(3, 0), Fixed::new(2, 0));
        player.body.collision_size = vec3(Fixed::new(0, 128), Fixed::new(2, 0), Fixed::new(0, 128));
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

        if self.is_camera_first_person {
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
            // TODO: Adjust camera z planes and player hitbox so camera doesn't clip into walls
            // when you walk up to them
            self.camera.position = (Fixed::point3_to_f32(self.player.body.position) + vec3(0.0, 1.2, 0.0)) * VOXEL_SIZE.x;
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
            self.camera.target = Fixed::point3_to_f32(self.player.body.position) * VOXEL_SIZE.x;
            self.camera.position = self.orbit_camera_controller.get_camera_position(&self.camera.target);
        }
    }

    pub fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = vec![];
        vertices.append(&mut self.chunk.get_vertices().clone());
        vertices.append(&mut create_pyramid_mesh(Fixed::point3_to_f32(self.player.body.position) * VOXEL_SIZE.x, 0.25, 0.5));
        vertices
    }
}
