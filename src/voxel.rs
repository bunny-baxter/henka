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

const CHUNK_SIZE: Vector3<usize> = vec3(32, 32, 32);

pub struct VoxelChunk {
    voxels: Array3D,
    mesh_data: Option<Vec<Vertex>>,
}

impl VoxelChunk {
    pub fn new() -> Self {
        VoxelChunk {
            voxels: Array3D::new(CHUNK_SIZE),
            mesh_data: None,
        }
    }

    pub fn set_voxel(&mut self, coord: Vector3<usize>, value: i32) {
        self.voxels.set(coord, value);
    }

    pub fn create_vertices(&mut self) -> Vec<Vertex> {
        let face_description = CubeFaceDescription {
            render_posx_face: true,
            render_negx_face: true,
            render_posy_face: true,
            render_negy_face: true,
            render_posz_face: true,
            render_negz_face: true,
        };
        create_cube_mesh(vec3(0.0, 0.0, 0.0), vec3(0.5, 0.5, 0.5), face_description)
    }
}
