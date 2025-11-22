use cgmath::{Point3, point3, Vector3, vec3};

use crate::fixed_point::Fixed;
use crate::voxel::VoxelChunk;

pub struct PhysicsBody {
    pub position: Point3<Fixed>,
    pub velocity: Vector3<Fixed>,
    pub collision_size: Vector3<Fixed>,
    pub is_on_ground: bool,
}

impl PhysicsBody {
    pub fn new() -> Self {
        Self {
            position: Fixed::ZERO_POINT,
            velocity: Fixed::ZERO_VECTOR,
            collision_size: Fixed::ZERO_VECTOR,
            is_on_ground: false,
        }
    }

    pub fn has_collision(&self) -> bool {
        self.collision_size.x != Fixed::ZERO && self.collision_size.y != Fixed::ZERO && self.collision_size.z != Fixed::ZERO
    }

    pub fn collision_extent(&self) -> Point3<Fixed> {
        add_vec_to_point(self.position, add_vec_to_vec(self.collision_size, vec3(-Fixed::EPSILON, -Fixed::EPSILON, -Fixed::EPSILON)))
    }
}

pub struct PhysicsConfig {
    pub gravity: Vector3<Fixed>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        PhysicsConfig {
            gravity: Fixed::ZERO_VECTOR,
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

fn physics_to_voxel(component: Fixed) -> Option<usize> {
    let f = component.to_f32().floor();
    if f >= 0.0 { Some(f as usize) } else { None }
}

fn is_body_colliding_with_voxels(body: &PhysicsBody, voxels: &VoxelChunk) -> bool {
    if !body.has_collision() {
        return false;
    }
    let min_voxel_x = physics_to_voxel(body.position.x).unwrap_or(0);
    let min_voxel_y = physics_to_voxel(body.position.y).unwrap_or(0);
    let min_voxel_z = physics_to_voxel(body.position.z).unwrap_or(0);
    let extent = body.collision_extent();
    let max_voxel_x = physics_to_voxel(extent.x).unwrap_or(0);
    let max_voxel_y = physics_to_voxel(extent.y).unwrap_or(0);
    let max_voxel_z = physics_to_voxel(extent.z).unwrap_or(0);
    for x in min_voxel_x..=max_voxel_x {
        for y in min_voxel_y..=max_voxel_y {
            for z in min_voxel_z..=max_voxel_z {
                if voxels.get_voxel(vec3(x, y, z)) != 0 {
                    return true;
                }
            }
        }
    }
    false
}

// TODO: Walking off the edge of the chunk in the positive direction panics.
// TODO: Walking off the edge of the chunk in the negative direction doesn't apply gravity.
// TODO: Probably should cap velocity to avoid arbitrarily large tick loops.

pub fn physics_tick(config: &PhysicsConfig, bodies: &mut [PhysicsBody], voxels: &VoxelChunk) {
    for body in bodies.iter_mut() {
        body.is_on_ground = false;
        body.velocity = add_vec_to_vec(body.velocity, config.gravity);
        for _x in 0..body.velocity.x.epsilons() {
            let previous = body.position.x;
            body.position.x += if body.velocity.x.is_negative() { -Fixed::EPSILON } else { Fixed::EPSILON };
            if is_body_colliding_with_voxels(body, voxels) {
                body.position.x = previous;
                body.velocity.x = Fixed::ZERO;
                break;
            }
        }
        for _y in 0..body.velocity.y.epsilons() {
            let previous = body.position.y;
            body.position.y += if body.velocity.y.is_negative() { -Fixed::EPSILON } else { Fixed::EPSILON };
            if is_body_colliding_with_voxels(body, voxels) {
                body.position.y = previous;
                body.velocity.y = Fixed::ZERO;
                body.is_on_ground = true;
                break;
            }
        }
        for _z in 0..body.velocity.z.epsilons() {
            let previous = body.position.z;
            body.position.z += if body.velocity.z.is_negative() { -Fixed::EPSILON } else { Fixed::EPSILON };
            if is_body_colliding_with_voxels(body, voxels) {
                body.position.z = previous;
                body.velocity.z = Fixed::ZERO;
                break;
            }
        }
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

        assert_eq!(Fixed::ZERO_POINT, bodies[0].position);
        assert_eq!(Fixed::ZERO_VECTOR, bodies[0].velocity);

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

    #[test]
    fn test_physics_to_voxel() {
        assert_eq!(Some(1), physics_to_voxel(Fixed::new(1, 0)));
        assert_eq!(Some(0), physics_to_voxel(Fixed::new(0, 255)));
        assert_eq!(None, physics_to_voxel(-Fixed::new(0, 1)));
    }

    #[test]
    fn fall_onto_ground() {
        let config = PhysicsConfig { gravity: vec3(Fixed::ZERO, -Fixed::new(0, 64), Fixed::ZERO) };
        let mut bodies = vec![ PhysicsBody::new() ];
        let mut voxel_chunk = VoxelChunk::new();

        bodies[0].position = point3(Fixed::ZERO, Fixed::new(2, 0), Fixed::ZERO);
        bodies[0].collision_size = vec3(Fixed::new(0, 64), Fixed::new(0, 64), Fixed::new(0, 64));
        voxel_chunk.set_voxel(vec3(0, 0, 0), 1);

        let ys = [Fixed::new(1, 192), Fixed::new(1, 64), Fixed::new(1, 0), Fixed::new(1, 0)];
        for &y in ys.iter() {
            physics_tick(&config, &mut bodies, &voxel_chunk);
            assert_eq!(point3(Fixed::ZERO, y, Fixed::ZERO), bodies[0].position);
        }
    }

    #[test]
    fn run_off_an_edge() {
        let config = PhysicsConfig { gravity: vec3(Fixed::ZERO, -Fixed::new(0, 64), Fixed::ZERO) };
        let mut bodies = vec![ PhysicsBody::new() ];
        let mut voxel_chunk = VoxelChunk::new();

        bodies[0].position = point3(Fixed::ZERO, Fixed::new(1, 0), Fixed::ZERO);
        bodies[0].velocity = vec3(Fixed::new(0, 86), Fixed::ZERO, Fixed::ZERO);
        bodies[0].collision_size = vec3(Fixed::new(0, 64), Fixed::new(0, 64), Fixed::new(0, 64));
        voxel_chunk.set_voxel(vec3(0, 0, 0), 1);

        let xys = [
            (Fixed::new(0, 86), Fixed::new(1, 0)),
            (Fixed::new(0, 172), Fixed::new(1, 0)),
            (Fixed::new(1, 2), Fixed::new(0, 192)), // Falls off; gravity begins applying
            (Fixed::new(1, 88), Fixed::new(0, 64)),
        ];
        for &(x, y) in xys.iter() {
            physics_tick(&config, &mut bodies, &voxel_chunk);
            assert_eq!(point3(x, y, Fixed::ZERO), bodies[0].position);
        }
    }

    #[test]
    fn run_into_wall() {
        let config = PhysicsConfig::default();
        let mut bodies = vec![ PhysicsBody::new() ];
        let mut voxel_chunk = VoxelChunk::new();

        bodies[0].collision_size = vec3(Fixed::new(0, 64), Fixed::new(0, 64), Fixed::new(0, 64));
        bodies[0].velocity = vec3(Fixed::new(0, 128), Fixed::ZERO, Fixed::new(0, 128));
        voxel_chunk.set_voxel(vec3(2, 0, 1), 1);
        voxel_chunk.set_voxel(vec3(2, 0, 2), 1);
        voxel_chunk.set_voxel(vec3(2, 0, 3), 1);
        voxel_chunk.set_voxel(vec3(1, 0, 3), 1);

        let xzs = [
            (Fixed::new(0, 128), Fixed::new(0, 128)),
            (Fixed::new(1, 0), Fixed::new(1, 0)),
            (Fixed::new(1, 128), Fixed::new(1, 128)),
            (Fixed::new(1, 192), Fixed::new(2, 0)), // Collides with wall and begins sliding.
            (Fixed::new(1, 192), Fixed::new(2, 128)),
            (Fixed::new(1, 192), Fixed::new(2, 192)), // Collides with corner.
            (Fixed::new(1, 192), Fixed::new(2, 192)),
        ];
        for &(x, z) in xzs.iter() {
            physics_tick(&config, &mut bodies, &voxel_chunk);
            assert_eq!(point3(x, Fixed::ZERO, z), bodies[0].position);
        }
    }
}
