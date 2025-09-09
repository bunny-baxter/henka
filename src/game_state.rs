use cgmath::{point3, Vector2, vec3};

use crate::voxel::VoxelChunk;
use crate::camera::Camera;

pub struct GameState {
    pub t: i32,
    pub chunk: VoxelChunk,
    pub camera: Camera,
}

impl GameState {
    pub fn new(window_size: Vector2<u32>) -> Self {
        let aspect_ratio = window_size.x as f32 / window_size.y as f32;
        GameState {
            t: 0,
            chunk: VoxelChunk::new(),
            camera: Camera::new(point3(-2.0, 0.0, 2.0), point3(0.25, 0.25, 0.25), aspect_ratio),
        }
    }

    pub fn generate_voxels(&mut self) {
        self.chunk.set_voxel(vec3(0, 0, 0), 1);
        self.chunk.set_voxel(vec3(1, 0, 0), 1);
        self.chunk.set_voxel(vec3(0, 0, 1), 1);
        self.chunk.set_voxel(vec3(1, 0, 1), 1);
        self.chunk.set_voxel(vec3(0, 1, 0), 1);
    }

    pub fn update(&mut self) {
        self.t += 1;
        let orbit_t = self.t as f32 * 0.01;
        self.camera.position = point3(
            self.camera.target.x + orbit_t.cos() * 0.8,
            self.camera.target.y + 0.9,
            self.camera.target.z + orbit_t.sin() * 0.8,
        );
    }
}
