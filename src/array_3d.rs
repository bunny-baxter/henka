use cgmath::{Vector3, vec3};

pub fn vec_i32_as_usize(v: Vector3<i32>) -> Vector3<usize> {
    vec3(v.x.try_into().unwrap(), v.y.try_into().unwrap(), v.z.try_into().unwrap())
}

#[derive(Clone)]
pub struct Array3D {
    pub size: Vector3<usize>,
    data: Vec<i32>,
}

impl Array3D {
    pub fn new(size: Vector3<usize>) -> Self {
        let flat_size = size.x * size.y * size.z;
        Array3D {
            size: size,
            data: vec![ 0 ; flat_size ],
        }
    }

    fn coord_to_index(&self, coord: Vector3<usize>) -> usize {
        coord.z * self.size.x * self.size.y + coord.y * self.size.x + coord.x
    }

    fn index_to_coord(&self, index: usize) -> Vector3<usize> {
        let z = index / (self.size.x * self.size.y);
        let remainder = index % (self.size.x * self.size.y);
        let y = remainder / self.size.x;
        let x = remainder % self.size.x;
        vec3(x, y, z)
    }

    pub fn is_out_of_bounds(&self, coord: Vector3<usize>) -> bool {
        coord.x >= self.size.x || coord.y >= self.size.y || coord.z >= self.size.z
    }

    pub fn is_i32_out_of_bounds(&self, coord: Vector3<i32>) -> bool {
        if coord.x < 0 || coord.y < 0 || coord.z < 0 {
            return true;
        }
        self.is_out_of_bounds(vec_i32_as_usize(coord))
    }

    pub fn get(&self, coord: Vector3<usize>) -> i32 {
        self.data[self.coord_to_index(coord)]
    }

    pub fn get_i32(&self, coord: Vector3<i32>) -> i32 {
        self.get(vec_i32_as_usize(coord))
    }

    pub fn set(&mut self, coord: Vector3<usize>, value: i32) {
        let index = self.coord_to_index(coord);
        self.data[index] = value;
    }

    pub fn set_i32(&mut self, coord: Vector3<i32>, value: i32) {
        self.set(vec_i32_as_usize(coord), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_array() {
        let a = Array3D::new(vec3(3, 4, 5));
        assert_eq!(vec3(3, 4, 5), a.size);
        assert_eq!(60, a.data.len());
    }

    #[test]
    fn test_coord_to_index() {
        let a = Array3D::new(vec3(5, 4, 3));
        assert_eq!(0, a.coord_to_index(vec3(0, 0, 0)));
        assert_eq!(33, a.coord_to_index(vec3(3, 2, 1)));
        assert_eq!(59, a.coord_to_index(vec3(4, 3, 2)));
    }

    #[test]
    fn test_index_to_coord() {
        let a = Array3D::new(vec3(5, 4, 3));
        assert_eq!(vec3(0, 0, 0), a.index_to_coord(0));
        assert_eq!(vec3(3, 2, 1), a.index_to_coord(33));
        assert_eq!(vec3(4, 3, 2), a.index_to_coord(59));
    }

    #[test]
    fn test_out_of_bounds() {
        let a = Array3D::new(vec3(2, 2, 2));
        assert!(!a.is_out_of_bounds(vec3(0, 0, 0)));
        assert!(a.is_out_of_bounds(vec3(0, 0, 2)));
        assert!(a.is_out_of_bounds(vec3(0, 2, 0)));
        assert!(a.is_out_of_bounds(vec3(2, 0, 0)));

        assert!(!a.is_i32_out_of_bounds(vec3(0, 0, 0)));
        assert!(a.is_i32_out_of_bounds(vec3(0, 0, 2)));
        assert!(a.is_i32_out_of_bounds(vec3(0, 2, 0)));
        assert!(a.is_i32_out_of_bounds(vec3(2, 0, 0)));
        assert!(a.is_i32_out_of_bounds(vec3(-1, 0, 0)));
        assert!(a.is_i32_out_of_bounds(vec3(0, -1, 0)));
        assert!(a.is_i32_out_of_bounds(vec3(0, 0, -1)));
    }

    #[test]
    fn test_set_and_get() {
        let mut a = Array3D::new(vec3(3, 3, 3));
        a.set(vec3(0, 0, 0), 4);
        assert_eq!(4, a.get(vec3(0, 0, 0)));

        a.set_i32(vec3(0, 1, 2), 9);
        assert_eq!(9, a.get_i32(vec3(0, 1, 2)));
    }
}
