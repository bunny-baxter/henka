use cgmath::{Point3, point3, Vector3, vec3};

use crate::fixed_point::Fixed;
use crate::voxel::VoxelChunk;

pub struct PhysicsBody {
    pub position: Point3<Fixed>,
    pub velocity: Vector3<Fixed>,
}

impl PhysicsBody {
    pub fn new() -> Self {
        Self {
            position: point3(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
            velocity: vec3(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
        }
    }
}

pub struct PhysicsConfig {
    pub gravity: Vector3<Fixed>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        PhysicsConfig {
            gravity: vec3(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
        }
    }
}

// TODO: Looks like cgmath requires you implement *all* the math functions on the inner type before
// you can use the vector math functions. So we should only need these custom add functions until
// we get around to implementing multiplication and division.

fn add_vec_to_vec(v1: Vector3<Fixed>, v2: Vector3<Fixed>) -> Vector3<Fixed> {
    vec3(v1.x + v2.x, v1.y + v2.y, v1.z + v2.z)
}

fn add_vec_to_point(p: Point3<Fixed>, v: Vector3<Fixed>) -> Point3<Fixed> {
    point3(p.x + v.x, p.y + v.y, p.z + v.z)
}

pub fn physics_tick(config: &PhysicsConfig, bodies: &mut [PhysicsBody], voxels: &VoxelChunk) {
    for body in bodies.iter_mut() {
        body.velocity = add_vec_to_vec(body.velocity, config.gravity);
        body.position = add_vec_to_point(body.position, body.velocity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_movement() {
        let config = PhysicsConfig::default();
        let mut bodies = vec![ PhysicsBody::new() ];
        let voxel_chunk = VoxelChunk::new();

        assert_eq!(point3(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO), bodies[0].position);
        assert_eq!(vec3(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO), bodies[0].velocity);

        bodies[0].velocity.x = Fixed::new(1, 0);
        physics_tick(&config, &mut bodies, &voxel_chunk);
        assert_eq!(point3(Fixed::new(1, 0), Fixed::ZERO, Fixed::ZERO), bodies[0].position);

        bodies[0].velocity.x = Fixed::ZERO;
        bodies[0].velocity.y = Fixed::new(0, 128);
        physics_tick(&config, &mut bodies, &voxel_chunk);
        assert_eq!(point3(Fixed::new(1, 0), Fixed::new(0, 128), Fixed::ZERO), bodies[0].position);
    }

    #[test]
    fn gravity() {
        let config = PhysicsConfig { gravity: vec3(Fixed::ZERO, Fixed::new(-2, 0), Fixed::ZERO) };
        let mut bodies = vec![ PhysicsBody::new() ];
        let voxel_chunk = VoxelChunk::new();

        let ys = [-2, -6, -12, -20];
        for &y in ys.iter() {
            physics_tick(&config, &mut bodies, &voxel_chunk);
            assert_eq!(point3(Fixed::ZERO, Fixed::new(y, 0), Fixed::ZERO), bodies[0].position);
        }
    }
}
