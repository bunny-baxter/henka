use std::collections::HashSet;

use cgmath::{Point3, point3, Vector3, vec3};
use rand::Rng;

use crate::voxel::VoxelChunk;

pub struct EcosimEntity {
    pub voxel_coord: Vector3<usize>,
    pub genome: u32,
}

impl EcosimEntity {
    pub fn new(voxel_coord: Vector3<usize>) -> Self {
        EcosimEntity {
            voxel_coord,
            genome: 0,
        }
    }

    pub fn randomize_genome(&mut self) {
        let mut rng = rand::rng();
        self.genome = rng.random();
    }
}

pub fn flower_get_sprite_index(genome: u32) -> (u32, u32) {
    let x = if genome & 0b1 > 0 { 1 } else { 0 };
    let y = if genome & 0b10 > 0 { 1 } else { 0 };
    (x, y)
}

const ADJACENCIES: &[(i32, i32, i32)] = &[
    (1, 0, 0),
    (0, 1, 0),
    (0, 0, 1),
    (-1, 0, 0),
    (0, -1, 0),
    (0, 0, -1),

    (1, 1, 0),
    (1, -1, 0),
    (1, 0, 1),
    (1, 0, -1),
    (-1, 1, 0),
    (-1, -1, 0),
    (-1, 0, 1),
    (-1, 0, -1),
    (0, 1, 1),
    (0, 1, -1),
    (0, -1, 1),
    (0, -1, -1),
];

fn can_entity_grow_into_coord(coord: Vector3<i32>, voxels: &VoxelChunk) -> bool {
    if voxels.is_i32_out_of_bounds(coord) {
        return false;
    }
    let below_coord = vec3(coord.x, coord.y - 1, coord.z);
    if voxels.is_i32_out_of_bounds(below_coord) {
        return false;
    }
    voxels.get_voxel_i32(coord) == 0 && voxels.get_voxel_i32(below_coord) == 1
}

pub fn ecosim_tick(entities: &mut [EcosimEntity], voxels: &VoxelChunk) -> Vec<EcosimEntity> {
    let mut rng = rand::rng();
    let mut new_entities = vec![];
    let mut occupied_coords: HashSet<Vector3<i32>> = HashSet::new();
    for entity in entities.iter() {
        let coord_i32 = vec3(entity.voxel_coord.x as i32, entity.voxel_coord.y as i32, entity.voxel_coord.z as i32);
        occupied_coords.insert(coord_i32);
    }
    for entity in entities.iter() {
        let coord_i32 = vec3(entity.voxel_coord.x as i32, entity.voxel_coord.y as i32, entity.voxel_coord.z as i32);
        for &(dx, dy, dz) in ADJACENCIES.iter() {
            let adj = coord_i32 + vec3(dx, dy, dz);
            if can_entity_grow_into_coord(adj, voxels) && rng.random::<f32>() < 0.02 && !occupied_coords.contains(&adj) {
                new_entities.push(EcosimEntity {
                    voxel_coord: adj.map(|i| i as usize),
                    genome: entity.genome,
                });
                occupied_coords.insert(adj);
            }
        }
    }
    new_entities
}
