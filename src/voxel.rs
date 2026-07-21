use cgmath::{ElementWise, Vector3, vec3};

use crate::array_3d::Array3D;
use crate::render_util::Vertex;

fn vec_abs(v: Vector3<f32>) -> Vector3<f32> {
    vec3(v.x.abs(), v.y.abs(), v.z.abs())
}

fn vec_floor(v: Vector3<f32>) -> Vector3<f32> {
    vec3(v.x.floor(), v.y.floor(), v.z.floor())
}

fn vec_min(v: Vector3<f32>, limit: Vector3<f32>) -> Vector3<f32> {
    vec3(v.x.min(limit.x), v.y.min(limit.y), v.z.min(limit.z))
}

fn vec_max(v: Vector3<f32>, limit: Vector3<f32>) -> Vector3<f32> {
    vec3(v.x.max(limit.x), v.y.max(limit.y), v.z.max(limit.z))
}

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
pub const VOXEL_SIZE: Vector3<f32> = vec3(VOXEL_SCALE, VOXEL_SCALE, VOXEL_SCALE);

const RAYCAST_GRID_MIN: Vector3<f32> = vec3(0.0, 0.0, 0.0);
const RAYCAST_GRID_MAX: Vector3<f32> = vec3(CHUNK_SIZE.x as f32 * VOXEL_SIZE.x, CHUNK_SIZE.y as f32 * VOXEL_SIZE.y, CHUNK_SIZE.z as f32 * VOXEL_SIZE.z);

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

    pub fn is_face_visible(&self, voxel_position: Vector3<i32>, face_direction: Vector3<i32>) -> bool {
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

    pub fn set_voxel_face_light(&mut self, coord: Vector3<usize>, face_normal: [f32; 3], light: [f32; 3]) {
        for vert in self.per_voxel_vertices.get_mut(coord) {
            if vert.normal == face_normal {
                vert.light = light;
            }
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

    fn raycast_against_bounding_box(&self, box_min: Vector3<f32>, box_max: Vector3<f32>, ray_origin: Vector3<f32>, ray_direction: Vector3<f32>) -> Option<f32> {
        let mut t_min = 0.0;
        let mut t_max = f32::INFINITY;

        for axis in 0..3 {
            let t1 = (box_min[axis] - ray_origin[axis]) / ray_direction[axis];
            let t2 = (box_max[axis] - ray_origin[axis]) / ray_direction[axis];

            let d_min = t1.min(t2);
            let d_max = t1.max(t2);

            t_min = d_min.max(t_min);
            t_max = d_max.min(t_max);
        }

        if t_max >= t_min {
            Some(t_min)
        } else {
            None  // Miss bounding box
        }
    }

    pub fn raycast(&self, ray_origin: Vector3<f32>, ray_direction: Vector3<f32>) -> Option<f32> {
        const MAX_STEPS: usize = 128;
        let maybe_entry_t = self.raycast_against_bounding_box(RAYCAST_GRID_MIN, RAYCAST_GRID_MAX, ray_origin, ray_direction);

        let entry_t = match maybe_entry_t {
            Some(entry_t) => entry_t,
            None => return None,
        };
        let origin_inside_grid =
            ray_origin.x >= RAYCAST_GRID_MIN.x && ray_origin.x <= RAYCAST_GRID_MAX.x &&
            ray_origin.y >= RAYCAST_GRID_MIN.y && ray_origin.y <= RAYCAST_GRID_MAX.y &&
            ray_origin.z >= RAYCAST_GRID_MIN.z && ray_origin.z <= RAYCAST_GRID_MAX.z;
        let entry_position: Vector3<f32> = ((ray_origin + (ray_direction * (entry_t + 0.0001))) - RAYCAST_GRID_MIN).div_element_wise(VOXEL_SIZE);

        let step: Vector3<f32> = vec3(ray_direction.x.signum(), ray_direction.y.signum(), ray_direction.z.signum());
        let delta: Vector3<f32> = vec_abs(1.0 / ray_direction);

        let position_f32: Vector3<f32> = vec_min(vec_max(
            vec_floor(entry_position),
            vec3(0.0, 0.0, 0.0)), vec3(CHUNK_SIZE.x as f32, CHUNK_SIZE.y as f32, CHUNK_SIZE.z as f32));
        let mut t_max: Vector3<f32> = (position_f32 - entry_position + vec_max(step, vec3(0.0, 0.0, 0.0))).div_element_wise(ray_direction);
        let mut last_axis = 0;
        let mut position = vec3(position_f32.x as i32, position_f32.y as i32, position_f32.z as i32);

        for i in 0..MAX_STEPS {
            let voxel = self.get_voxel_i32(position);
            if voxel != 0 && !(i == 0 && origin_inside_grid) {
                // Hit
                return Some(match i {
                    0 => entry_t,
                    _ => entry_t + (t_max[last_axis] - delta[last_axis]) * VOXEL_SIZE[last_axis],
                });
            }

            if t_max.x < t_max.y {
                if t_max.x < t_max.z {
                    position.x += step.x as i32;
                    if position.x < 0 || position.x >= CHUNK_SIZE.x as i32 {
                        break;
                    }
                    last_axis = 0;
                    t_max.x += delta.x;
                } else {
                    position.z += step.z as i32;
                    if position.z < 0 || position.z >= CHUNK_SIZE.z as i32{
                        break;
                    }
                    last_axis = 2;
                    t_max.z += delta.z;
                }
            } else {
                if t_max.y < t_max.z {
                    position.y += step.y as i32;
                    if position.y < 0 || position.y >= CHUNK_SIZE.y as i32 {
                        break;
                    }
                    last_axis = 1;
                    t_max.y += delta.y;
                } else {
                    position.z += step.z as i32;
                    if position.z < 0 || position.z >= CHUNK_SIZE.z as i32 {
                        break;
                    }
                    last_axis = 2;
                    t_max.z += delta.z;
                }
            }
        }

        return None;  // Miss
    }
}
