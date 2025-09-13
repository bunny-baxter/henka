use std::cell::{Ref, RefCell};

use cgmath::{Vector3, vec3};

use crate::array_3d::Array3D;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

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
        verts.push(Vertex { position: positions[0], color: [1.0, 0.0, 0.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[1], color: [1.0, 0.0, 0.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[4], color: [1.0, 0.0, 0.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[0], color: [1.0, 0.0, 0.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[4], color: [1.0, 0.0, 0.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[2], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0] });
        //for _i in range(6):
        //    normals.append(Vector3(-1.0, 0.0, 0.0))
    }
    if face_description.render_negy_face {
        verts.push(Vertex { position: positions[0], color: [0.0, 1.0, 0.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[5], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[1], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[0], color: [0.0, 1.0, 0.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[3], color: [0.0, 1.0, 0.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[5], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] });
        //for _i in range(6):
        //    normals.append(Vector3(0.0, -1.0, 0.0))
    }
    if face_description.render_negz_face {
        verts.push(Vertex { position: positions[0], color: [0.0, 0.0, 1.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[6], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[3], color: [0.0, 0.0, 1.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[0], color: [0.0, 0.0, 1.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[2], color: [0.0, 0.0, 1.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[6], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0] });
        //for _i in range(6):
        //    normals.append(Vector3(0.0, 0.0, -1.0))
    }
    if face_description.render_posx_face {
        verts.push(Vertex { position: positions[7], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[3], color: [1.0, 0.0, 0.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[6], color: [1.0, 0.0, 0.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[7], color: [1.0, 0.0, 0.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[5], color: [1.0, 0.0, 0.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[3], color: [1.0, 0.0, 0.0], uv: [1.0, 1.0] });
        //for _i in range(6):
        //    normals.append(Vector3(1.0, 0.0, 0.0))
    }
    if face_description.render_posy_face {
        verts.push(Vertex { position: positions[7], color: [0.0, 1.0, 0.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[6], color: [0.0, 1.0, 0.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[2], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[7], color: [0.0, 1.0, 0.0], uv: [1.0, 1.0] });
        verts.push(Vertex { position: positions[2], color: [0.0, 1.0, 0.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[4], color: [0.0, 1.0, 0.0], uv: [0.0, 1.0] });
        //for _i in range(6):
        //    normals.append(Vector3(0.0, 1.0, 0.0))
    }
    if face_description.render_posz_face {
        verts.push(Vertex { position: positions[7], color: [0.0, 0.0, 1.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[4], color: [0.0, 0.0, 1.0], uv: [0.0, 0.0] });
        verts.push(Vertex { position: positions[1], color: [0.0, 0.0, 1.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[7], color: [0.0, 0.0, 1.0], uv: [1.0, 0.0] });
        verts.push(Vertex { position: positions[1], color: [0.0, 0.0, 1.0], uv: [0.0, 1.0] });
        verts.push(Vertex { position: positions[5], color: [0.0, 0.0, 1.0], uv: [1.0, 1.0] });
        //for _i in range(6):
        //    normals.append(Vector3(0.0, 0.0, 1.0))
    }
    verts
}

pub const CHUNK_SIZE: Vector3<usize> = vec3(32, 32, 32);

const VOXEL_SIZE: Vector3<f32> = vec3(0.5, 0.5, 0.5);

pub struct VoxelChunk {
    voxels: Array3D,
    cached_vertices: RefCell<Option<Vec<Vertex>>>,
}

impl VoxelChunk {
    pub fn new() -> Self {
        VoxelChunk {
            voxels: Array3D::new(CHUNK_SIZE),
            cached_vertices: None.into(),
        }
    }

    fn clear_cached_vertices(&mut self) {
        *self.cached_vertices.borrow_mut() = None;
    }

    pub fn set_voxel(&mut self, coord: Vector3<usize>, value: i32) {
        self.clear_cached_vertices();
        self.voxels.set(coord, value);
    }

    fn is_face_visible(&self, voxel_position: Vector3<i32>, face_direction: Vector3<i32>) -> bool {
        let adjacent_position = voxel_position + face_direction;
        if self.voxels.is_i32_out_of_bounds(adjacent_position) {
            return true;
        }
        return self.voxels.get_i32(adjacent_position) == 0;
    }

    fn create_vertices(&self) -> Vec<Vertex> {
        let mut result = vec![];
        for i in 0..self.voxels.size.x as i32 {
            for j in 0..self.voxels.size.y as i32 {
                for k in 0..self.voxels.size.z as i32 {
                    let coord = vec3(i, j, k);
                    if self.voxels.get_i32(coord) > 0 {
                        let offset = vec3(coord.x as f32 * VOXEL_SIZE.x, coord.y as f32 * VOXEL_SIZE.y, coord.z as f32 * VOXEL_SIZE.z);
                        let face_description = CubeFaceDescription {
                            render_posx_face: self.is_face_visible(coord, vec3(1, 0, 0)),
                            render_negx_face: self.is_face_visible(coord, vec3(-1, 0, 0)),
                            render_posy_face: self.is_face_visible(coord, vec3(0, 1, 0)),
                            render_negy_face: self.is_face_visible(coord, vec3(0, -1, 0)),
                            render_posz_face: self.is_face_visible(coord, vec3(0, 0, 1)),
                            render_negz_face: self.is_face_visible(coord, vec3(0, 0, -1)),
                        };
                        result.extend(create_cube_mesh(offset, VOXEL_SIZE, face_description));
                    }
                }
            }
        }
        result
    }

    pub fn get_vertices(&self) -> Ref<Vec<Vertex>> {
        if self.cached_vertices.borrow().is_none() {
            *self.cached_vertices.borrow_mut() = Some(self.create_vertices());
        }
        Ref::map(self.cached_vertices.borrow(), |option| option.as_ref().unwrap())
    }
}
