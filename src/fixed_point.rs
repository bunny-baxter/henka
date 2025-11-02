use std::fmt;
use std::ops;

use cgmath::Vector3;

const FRACTION_BITS: u32 = 8;
const WHOLE_BITS: u32 = 23;

pub const DENOMINATOR: u32 = 2u32.pow(FRACTION_BITS);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Fixed(u32);

impl Fixed {
    pub const ZERO: Fixed = Fixed(0);
    pub const EPSILON: Fixed = Fixed(1);

    pub fn new(whole: i32, fraction: u32) -> Self {
        assert!(whole < 2i32.pow(WHOLE_BITS));
        assert!(fraction < DENOMINATOR);
        let sign_bit = if whole < 0 { 1 } else { 0 } as u32;
        Fixed((sign_bit << 31) + ((whole.abs() as u32) << FRACTION_BITS) + fraction)
    }

    pub fn from_f32(value: f32) -> Self {
        let whole = value.trunc() as i32;
        let fraction = ((value - value.trunc()) * DENOMINATOR as f32).round() as u32;
        Self::new(whole, fraction)
    }

    pub fn vector3_from_f32(v: Vector3<f32>) -> Vector3<Fixed> {
        Vector3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn vector3_to_f32(v: Vector3<Fixed>) -> Vector3<f32> {
        Vector3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn unpack(&self) -> (i32, u32) {
        let mut whole = ((0x7fffff00 & self.0) >> FRACTION_BITS) as i32;
        if (0x80000000 & self.0) > 0 {
            whole = -whole;
        }
        let fraction = 0xff & self.0;
        (whole, fraction)
    }

    pub fn to_f32(&self) -> f32 {
        let (whole, fraction) = self.unpack();
        whole as f32 + (fraction as f32 / DENOMINATOR as f32)
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (whole, fraction) = self.unpack();
        write!(f, "Fixed({}, {})", whole, fraction)
    }
}

impl From<f32> for Fixed {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl From<Fixed> for f32 {
    fn from(value: Fixed) -> Self {
        value.to_f32()
    }
}

impl ops::Add<Fixed> for Fixed {
    type Output = Fixed;

    fn add(self, rhs: Fixed) -> Fixed {
        let (lw, lf) = self.unpack();
        let (rw, rf) = rhs.unpack();
        let mut new_fraction = lf + rf;
        let mut overflow = 0;
        if new_fraction >= DENOMINATOR {
            new_fraction -= DENOMINATOR;
            overflow += 1;
        }
        Fixed::new(lw + rw + overflow, new_fraction)
    }
}

impl ops::AddAssign<Fixed> for Fixed {
    fn add_assign(&mut self, rhs: Fixed) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_unpack() {
        assert_eq!((128, 0), Fixed::new(128, 0).unpack());
        assert_eq!((2, 4), Fixed::new(2, 4).unpack());
        assert_eq!((-6, 5), Fixed::new(-6, 5).unpack());
    }

    #[test]
    fn to_f32() {
        assert_eq!(4.0, Fixed::new(4, 0).to_f32());
        assert_eq!(2.5, Fixed::new(2, 128).to_f32());
    }

    #[test]
    fn from_f32() {
        assert_eq!((2, 0), Fixed::from_f32(2.0).unpack());
        assert_eq!((4, 128), Fixed::from_f32(4.5).unpack());
        assert_eq!((4, 26), Fixed::from_f32(4.1).unpack());
    }

    #[test]
    fn add() {
        assert_eq!(Fixed::new(3, 0), Fixed::new(1, 0) + Fixed::new(2, 0));
        assert_eq!(Fixed::new(100, 10), Fixed::new(99, 0) + Fixed::new(1, 10));
        assert_eq!(Fixed::new(4, 4), Fixed::new(2, 2) + Fixed::new(2, 2));
        assert_eq!(Fixed::new(6, 0), Fixed::new(2, 128) + Fixed::new(3, 128));
        assert_eq!(Fixed::new(10, 11), Fixed::new(2, 255) + Fixed::new(7, 12));
    }

    #[test]
    fn epsilon() {
        let mut f = Fixed::ZERO;
        for _ in 0..DENOMINATOR {
            f += Fixed::EPSILON;
        }
        assert_eq!((1, 0), f.unpack());
    }

    #[test]
    fn into_from_f32() {
        assert_eq!(Fixed::new(2, 128), 2.5f32.into());
        assert_eq!(Fixed::new(4, 26), 4.1f32.into());
    }

    #[test]
    fn into_to_f32() {
        let f: f32 = Fixed::new(4, 0).into();
        assert_eq!(4.0f32, f);

        let f: f32 = Fixed::new(2, 128).into();
        assert_eq!(2.5f32, f);
    }

    #[test]
    fn vector3_from_f32() {
        let v_f32 = Vector3::new(1.5f32, 2.25f32, 3.75f32);
        let v_fixed = Fixed::vector3_from_f32(v_f32);

        assert_eq!((1, 128), v_fixed.x.unpack());
        assert_eq!((2, 64), v_fixed.y.unpack());
        assert_eq!((3, 192), v_fixed.z.unpack());
    }

    #[test]
    fn vector3_to_f32() {
        let v_fixed = Vector3::new(
            Fixed::new(1, 128),
            Fixed::new(2, 64),
            Fixed::new(3, 192),
        );
        let v_f32 = Fixed::vector3_to_f32(v_fixed);

        assert_eq!(1.5, v_f32.x);
        assert_eq!(2.25, v_f32.y);
        assert_eq!(3.75, v_f32.z);
    }

    #[test]
    fn debug_format() {
        assert_eq!("Fixed(2, 128)", format!("{:?}", Fixed::new(2, 128)));
        assert_eq!("Fixed(-6, 5)", format!("{:?}", Fixed::new(-6, 5)));
        assert_eq!("Fixed(0, 0)", format!("{:?}", Fixed::ZERO));
    }
}
