use cgmath::{point3, Vector2, vec2, vec3};
use winit::keyboard::KeyCode;

use crate::voxel::{CHUNK_SIZE, VoxelChunk};
use crate::camera::Camera;
use crate::window::InputState;

struct OrbitCameraController {
    t: i32,
    height: f32,
    zoom: f32,
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
                height: 0.6,
            },
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
        let orbit_t = self.camera_controller.t as f32 * 0.02;
        self.camera.position = point3(
            self.camera.target.x + orbit_t.cos() * self.camera_controller.zoom,
            self.camera.target.y + self.camera_controller.height,
            self.camera.target.z + orbit_t.sin() * self.camera_controller.zoom,
        );
    }
}
