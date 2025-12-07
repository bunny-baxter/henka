use std::collections::HashMap;

use cgmath::{Point3, point3, Vector3, vec3};
use rand::Rng;

use crate::fixed_point::Fixed;
use crate::voxel::VoxelChunk;

const FLOWER_MATURITY_AGE: u32 = 20;
const FLOWER_LIFESPAN: u32 = 120;

pub struct EcosimEntity {
    pub position: Point3<Fixed>,
    pub genome: u32,
    pub age_ticks: u32,
    pub stress: u32,
    pub dead_ticks: Option<u32>,
}

impl EcosimEntity {
    pub fn new(voxel_coord: Vector3<usize>) -> Self {
        let mut rng = rand::rng();
        EcosimEntity {
            position: point3(
                Fixed::new(voxel_coord.x as i32, rng.random_range(16..=240)),
                Fixed::new(voxel_coord.y as i32, 0),
                Fixed::new(voxel_coord.z as i32, rng.random_range(16..=240)),
            ),
            genome: 0,
            age_ticks: 0,
            stress: 0,
            dead_ticks: None,
        }
    }

    pub fn voxel_coord(&self) -> Vector3<i32> {
        vec3(
            self.position.x.to_f32().floor() as i32,
            self.position.y.to_f32().floor() as i32,
            self.position.z.to_f32().floor() as i32,
        )
    }

    pub fn randomize_genome(&mut self) {
        let mut rng = rand::rng();
        self.genome = rng.random();
    }

    fn mutate_genome(&mut self) {
        const MUTATION_RATE: f32 = 0.05;
        let mut rng = rand::rng();
        let mut result = self.genome;
        for i in 0..32 {
            if rng.random::<f32>() < MUTATION_RATE {
                result ^= 1 << i;
            }
        }
        self.genome = result;
    }

    pub fn flower_get_sprite_index(&self) -> (u32, u32) {
        let x = if self.age_ticks < FLOWER_MATURITY_AGE / 2 {
            0
        } else if self.age_ticks < FLOWER_MATURITY_AGE {
            1
        } else {
            2
        };
        let y = if self.dead_ticks.is_some() {
            4
        } else {
            let light = self.genome & 0b1 > 0;
            let color = self.genome & 0b10 > 0;
            if light { if color { 2 } else { 0 } } else { if color { 3 } else { 1 } }
        };
        (x, y)
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

pub fn ecosim_tick(entities: &mut Vec<EcosimEntity>, voxels: &VoxelChunk) {
    let mut rng = rand::rng();
    let mut new_entities = vec![];
    let mut coord_population: HashMap<Vector3<i32>, u32> = HashMap::new();

    // Increase age and calculate voxel populations
    for entity in entities.iter_mut() {
        match entity.dead_ticks {
            Some(ref mut dead_ticks) => *dead_ticks += 1,
            None => entity.age_ticks += 1,
        };
        if entity.age_ticks >= FLOWER_LIFESPAN && rng.random::<f32>() < 0.01 {
            entity.dead_ticks = Some(0);
        }
        if entity.dead_ticks.is_none() {
            *coord_population.entry(entity.voxel_coord()).or_insert(0) += 1;
        }
    }

    // Maybe reproduce
    for entity in entities.iter_mut() {
        let coord_i32 = entity.voxel_coord();
        for &(dx, dy, dz) in ADJACENCIES.iter() {
            let adj = coord_i32 + vec3(dx, dy, dz);
            if entity.dead_ticks.is_none() && entity.age_ticks >= FLOWER_MATURITY_AGE && *coord_population.get(&adj).unwrap_or(&0u32) < 6 && can_entity_grow_into_coord(adj, voxels) && rng.random::<f32>() < 0.006 {
                let mut new_entity = EcosimEntity::new(adj.map(|i| i as usize));
                new_entity.genome = entity.genome;
                new_entity.mutate_genome();
                new_entities.push(new_entity);
                *coord_population.entry(adj).or_insert(0) += 1;
                entity.stress += 200;
            }
        }
    }

    // Resolve stress
    for entity in entities.iter_mut() {
        if entity.dead_ticks.is_some() {
            continue;
        }
        let population = *coord_population.get(&entity.voxel_coord()).unwrap();
        if population >= 2 {
            let n = population - 1;
            entity.stress += n * n;
        }
        if (entity.age_ticks < FLOWER_MATURITY_AGE && entity.stress >= 200) || (entity.age_ticks >= FLOWER_MATURITY_AGE && entity.stress > 2000) {
            entity.dead_ticks = Some(0);
        }
    }

    entities.retain(|entity| entity.dead_ticks.is_none() || entity.dead_ticks.unwrap() < 8);
    entities.append(&mut new_entities);
}
