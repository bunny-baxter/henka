use cgmath::{Point3, point3, Vector2, vec2, Vector3, vec3};
use fixed::types::I24F8;
use winit::keyboard::KeyCode;

use crate::voxel::{CHUNK_SIZE, VoxelChunk, Vertex};
use crate::camera::Camera;
use crate::window::InputState;

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

struct PlayerActor {
    position: Point3<I24F8>,
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
    camera_controller: OrbitCameraController,
    player: PlayerActor,
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
            camera_controller: OrbitCameraController {
                t: 0,
                zoom: 1.4,
                height: 0.6,
            },
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
            _ => (),
        };
    }

    pub fn on_key_released(&mut self, key_code: KeyCode) {
        match key_code {
            _ => (),
        };
    }

    pub fn update(&mut self, input_state: &InputState) {
        if input_state.is_key_pressed(KeyCode::KeyW) | input_state.is_key_pressed(KeyCode::ArrowUp) {
            self.camera_controller.zoom -= 0.01;
        }
        if input_state.is_key_pressed(KeyCode::KeyS) | input_state.is_key_pressed(KeyCode::ArrowDown) {
            self.camera_controller.zoom += 0.01;
        }
        if self.camera_controller.zoom < 0.5 {
            self.camera_controller.zoom = 0.5;
        }
        if input_state.is_key_pressed(KeyCode::KeyD) | input_state.is_key_pressed(KeyCode::ArrowRight) {
            self.camera_controller.t += 1;
        }
        if input_state.is_key_pressed(KeyCode::KeyA) | input_state.is_key_pressed(KeyCode::ArrowLeft) {
            self.camera_controller.t -= 1;
        }
        if input_state.is_key_pressed(KeyCode::KeyJ) {
            self.camera_controller.height -= 0.05;
        }
        if input_state.is_key_pressed(KeyCode::KeyK) {
            self.camera_controller.height += 0.05;
        }
        self.player.position.z += I24F8::from_bits(1);
        self.camera.target = self.player.position.map(|i| i.to_num::<f32>());
        self.camera.position = self.camera_controller.get_camera_position(&self.camera.target);
    }

    pub fn get_vertices(&self) -> Vec<Vertex> {
        let mut vertices = vec![];
        vertices.append(&mut self.chunk.get_vertices().clone());
        let player_position_f32 = self.player.position.map(|i| i.to_num::<f32>());
        vertices.append(&mut create_pyramid_mesh(player_position_f32, 0.25, 0.5));
        vertices
    }
}
