use cgmath::{Vector3, vec3};

use crate::array_3d::Array3D;
use crate::render_util::Vertex;

type VoxelType = u32;

struct CubeFaceDescription {
    render_posx_face: bool,
    render_negx_face: bool,
    render_posy_face: bool,
    render_negy_face: bool,
    render_posz_face: bool,
    render_negz_face: bool,
}

fn create_cube_mesh(offset: Vector3<f32>, size: Vector3<f32>, face_description: CubeFaceDescription) -> Vec<Vertex> {
    //      +Y
    //       |
    //       2 -------- 6
    //      /|         /|
    //     / |        / |
    //    4 -------- 7  |
    //    |  |       |  |
    //    |  0 ------|- 3 --- +X
    //    | /        | /
    //    |/         |/
    //    1 -------- 5
    //   /
    // +Z
    let positions = [
        [offset.x, offset.y, offset.z],
        [offset.x, offset.y, offset.z + size.z],
        [offset.x, offset.y + size.y, offset.z],
        [offset.x + size.x, offset.y, offset.z],
        [offset.x, offset.y + size.y, offset.z + size.z],
        [offset.x + size.x, offset.y, offset.z + size.z],
        [offset.x + size.x, offset.y + size.y, offset.z],
        [offset.x + size.x, offset.y + size.y, offset.z + size.z],
    ];
    let mut verts = vec![];
    if face_description.render_negx_face {
        const NORMAL: [f32; 3] = [-1.0, 0.0, 0.0];
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[1], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[4], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[4], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[2], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
    }
    if face_description.render_negy_face {
        const NORMAL: [f32; 3] = [0.0, -1.0, 0.0];
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[5], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[1], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[3], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[5], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
    }
    if face_description.render_negz_face {
        const NORMAL: [f32; 3] = [0.0, 0.0, -1.0];
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[6], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[3], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[0], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[2], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[6], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
    }
    if face_description.render_posx_face {
        const NORMAL: [f32; 3] = [1.0, 0.0, 0.0];
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[3], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[6], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[5], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[3], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
    }
    if face_description.render_posy_face {
        const NORMAL: [f32; 3] = [0.0, 1.0, 0.0];
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[6], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[2], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[2], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[4], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
    }
    if face_description.render_posz_face {
        const NORMAL: [f32; 3] = [0.0, 0.0, 1.0];
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[4], light: [0.0, 0.0, 0.0], uv: [0.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[1], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[7], light: [0.0, 0.0, 0.0], uv: [1.0, 0.0], normal: NORMAL });
        verts.push(Vertex { position: positions[1], light: [0.0, 0.0, 0.0], uv: [0.0, 1.0], normal: NORMAL });
        verts.push(Vertex { position: positions[5], light: [0.0, 0.0, 0.0], uv: [1.0, 1.0], normal: NORMAL });
    }
    verts
}

pub const CHUNK_SIZE: Vector3<usize> = vec3(32, 32, 32);

pub const VOXEL_SCALE: f32 = 0.5;

const VOXEL_SIZE: Vector3<f32> = vec3(VOXEL_SCALE, VOXEL_SCALE, VOXEL_SCALE);

pub struct VoxelChunk {
    voxels: Array3D<VoxelType>,
    per_voxel_vertices: Array3D<Vec<Vertex>>,
    geometry_dirty: bool,
}

impl VoxelChunk {
    pub fn new() -> Self {
        VoxelChunk {
            voxels: Array3D::new(CHUNK_SIZE),
            per_voxel_vertices: Array3D::new(CHUNK_SIZE),
            geometry_dirty: true,
        }
    }

    pub fn is_i32_out_of_bounds(&self, coord: Vector3<i32>) -> bool {
        self.voxels.is_i32_out_of_bounds(coord)
    }

    pub fn get_voxel(&self, coord: Vector3<usize>) -> VoxelType {
        *self.voxels.get(coord)
    }

    pub fn get_voxel_i32(&self, coord: Vector3<i32>) -> VoxelType {
        *self.voxels.get_i32(coord)
    }

    pub fn set_voxel(&mut self, coord: Vector3<usize>, value: VoxelType) {
        self.voxels.set(coord, value);
        self.geometry_dirty = true;
    }

    fn is_face_visible(&self, voxel_position: Vector3<i32>, face_direction: Vector3<i32>) -> bool {
        let adjacent_position = voxel_position + face_direction;
        if self.voxels.is_i32_out_of_bounds(adjacent_position) {
            return true;
        }
        return *self.voxels.get_i32(adjacent_position) == 0;
    }

    fn create_voxel_vertices(&self, coord: Vector3<i32>) -> Vec<Vertex> {
        if *self.voxels.get_i32(coord) <= 0 {
            return vec![];
        }
        let offset = vec3(coord.x as f32 * VOXEL_SIZE.x, coord.y as f32 * VOXEL_SIZE.y, coord.z as f32 * VOXEL_SIZE.z);
        let face_description = CubeFaceDescription {
            render_posx_face: self.is_face_visible(coord, vec3(1, 0, 0)),
            render_negx_face: self.is_face_visible(coord, vec3(-1, 0, 0)),
            render_posy_face: self.is_face_visible(coord, vec3(0, 1, 0)),
            render_negy_face: self.is_face_visible(coord, vec3(0, -1, 0)),
            render_posz_face: self.is_face_visible(coord, vec3(0, 0, 1)),
            render_negz_face: self.is_face_visible(coord, vec3(0, 0, -1)),
        };
        create_cube_mesh(offset, VOXEL_SIZE, face_description)
    }

    fn rebuild_all_vertices(&mut self) {
        for i in 0..self.voxels.size.x as i32 {
            for j in 0..self.voxels.size.y as i32 {
                for k in 0..self.voxels.size.z as i32 {
                    let coord = vec3(i, j, k);
                    let verts = self.create_voxel_vertices(coord);
                    self.per_voxel_vertices.set_i32(coord, verts);
                }
            }
        }
        self.geometry_dirty = false;
    }

    pub fn set_voxel_light(&mut self, coord: Vector3<usize>, light: [f32; 3]) {
        for vert in self.per_voxel_vertices.get_mut(coord) {
            vert.light = light;
        }
    }

    pub fn get_vertices(&mut self) -> Vec<Vertex> {
        if self.geometry_dirty {
            self.rebuild_all_vertices();
        }
        let mut result = vec![];
        for i in 0..self.per_voxel_vertices.size.x {
            for j in 0..self.per_voxel_vertices.size.y {
                for k in 0..self.per_voxel_vertices.size.z {
                    result.extend_from_slice(self.per_voxel_vertices.get(vec3(i, j, k)));
                }
            }
        }
        result
    }
}
