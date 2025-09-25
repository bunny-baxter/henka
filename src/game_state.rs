use cgmath::{InnerSpace, Point3, point3, Vector2, vec2, Vector3, vec3};
use fixed::types::I24F8;
use winit::keyboard::KeyCode;

use crate::voxel::{CHUNK_SIZE, VoxelChunk, Vertex};
use crate::camera::Camera;
use crate::window::InputState;

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
    pub position: Point3<I24F8>,
}

impl PlayerActor {
    fn new() -> Self {
        PlayerActor {
            position: point3(I24F8::from_num(0), I24F8::from_num(0), I24F8::from_num(0)),
        }
    }
}

// Function written by Claude, cleaned up by me
fn create_pyramid_mesh(offset: Point3<f32>, base_size: f32, height: f32) -> Vec<Vertex> {
    const YELLOW: [f32; 3] = [1.0, 1.0, 0.0];
    let half_base = base_size / 2.0;

    // Define the 5 unique vertices of the pyramid
    // Apex (top point)
    let apex = Vertex {
        position: [offset.x, offset.y + height, offset.z],
        color: YELLOW,
        uv: [0.5, 0.5],
    };
    // Base vertices (counter-clockwise when viewed from above)
    let base_v0 = Vertex {
        position: [offset.x - half_base, offset.y, offset.z - half_base],
        color: YELLOW,
        uv: [0.0, 0.0],
    };
    let base_v1 = Vertex {
        position: [offset.x + half_base, offset.y, offset.z - half_base],
        color: YELLOW,
        uv: [1.0, 0.0],
    };
    let base_v2 = Vertex {
        position: [offset.x + half_base, offset.y, offset.z + half_base],
        color: YELLOW,
        uv: [1.0, 1.0],
    };
    let base_v3 = Vertex {
        position: [offset.x - half_base, offset.y, offset.z + half_base],
        color: YELLOW,
        uv: [0.0, 1.0],
    };

    // Create triangles for the pyramid
    // 4 triangular faces + 2 triangles for the square base = 18 vertices total
    vec![
        // Front face
        base_v0, apex, base_v1,
        // Right face
        base_v1, apex, base_v2,
        // Back face
        base_v2, apex, base_v3,
        // Left face
        base_v3, apex, base_v0,
        // Base (two triangles)
        base_v0, base_v1, base_v2,
        base_v0, base_v2, base_v3,
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
    pub player: PlayerActor,
}

impl GameState {
    pub fn new() -> Self {
        let mut player = PlayerActor::new();
        player.position = point3(I24F8::from_num(1.0), I24F8::from_num(1.5), I24F8::from_num(0.0));
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
            player,
        }
    }

    pub fn set_window_size(&mut self, window_size: Vector2<u32>) {
        self.window_size = window_size;
        self.camera.aspect_ratio = window_size.x as f32 / window_size.y as f32;
    }

    pub fn generate_voxels(&mut self) {
        for i in 0..CHUNK_SIZE.x {
            for k in 0..CHUNK_SIZE.z {
                for j in 0..3 {
                    self.chunk.set_voxel(vec3(i, j, k), 1);
                }
            }
        }
        self.chunk.set_voxel(vec3(4, 4, 4), 1);
    }

    pub fn on_key_pressed(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::KeyQ => self.exit = true,
            KeyCode::KeyC => self.is_camera_first_person = !self.is_camera_first_person,
            _ => (),
        };
    }

    pub fn update(&mut self, input_state: &InputState) {
        if self.is_camera_first_person {
            let forward = self.first_person_camera_controller.get_forward();
            let forward_fixed = forward.map(|i| I24F8::from_num(i * 0.01));
            let right = forward.cross(cgmath::Vector3::unit_y()).normalize();
            let right_fixed = right.map(|i| I24F8::from_num(i * 0.01));
            if input_state.is_key_pressed(KeyCode::KeyW) {
                self.player.position.x += forward_fixed.x;
                self.player.position.z += forward_fixed.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyS) {
                self.player.position.x -= forward_fixed.x;
                self.player.position.z -= forward_fixed.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyD) {
                self.player.position.x += right_fixed.x;
                self.player.position.z += right_fixed.z;
            }
            if input_state.is_key_pressed(KeyCode::KeyA) {
                self.player.position.x -= right_fixed.x;
                self.player.position.z -= right_fixed.z;
            }
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
            self.camera.position = self.player.position.map(|i| i.to_num::<f32>());
            self.camera.position.y += 0.4;
            self.camera.target = self.first_person_camera_controller.get_camera_target(&self.camera.position);
        } else {
            if input_state.is_key_pressed(KeyCode::KeyW) | input_state.is_key_pressed(KeyCode::ArrowUp) {
                self.orbit_camera_controller.zoom -= 0.01;
            }
            if input_state.is_key_pressed(KeyCode::KeyS) | input_state.is_key_pressed(KeyCode::ArrowDown) {
                self.orbit_camera_controller.zoom += 0.01;
            }
            if self.orbit_camera_controller.zoom < 0.5 {
                self.orbit_camera_controller.zoom = 0.5;
            }
            if input_state.is_key_pressed(KeyCode::KeyD) | input_state.is_key_pressed(KeyCode::ArrowRight) {
                self.orbit_camera_controller.t += 1;
            }
            if input_state.is_key_pressed(KeyCode::KeyA) | input_state.is_key_pressed(KeyCode::ArrowLeft) {
                self.orbit_camera_controller.t -= 1;
            }
            if input_state.is_key_pressed(KeyCode::KeyJ) {
                self.orbit_camera_controller.height -= 0.05;
            }
            if input_state.is_key_pressed(KeyCode::KeyK) {
                self.orbit_camera_controller.height += 0.05;
            }
            self.camera.target = self.player.position.map(|i| i.to_num::<f32>());
            self.camera.position = self.orbit_camera_controller.get_camera_position(&self.camera.target);
        }
    }

    pub fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = vec![];
        vertices.append(&mut self.chunk.get_vertices().clone());
        let player_position_f32 = self.player.position.map(|i| i.to_num::<f32>());
        vertices.append(&mut create_pyramid_mesh(player_position_f32, 0.25, 0.5));
        vertices
    }
}
