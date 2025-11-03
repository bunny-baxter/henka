use std::fmt;
use std::ops;

use cgmath::{Point3, Vector3};

const FRACTION_BITS: u32 = 8;
const WHOLE_BITS: u32 = 23;

pub const DENOMINATOR: u32 = 2u32.pow(FRACTION_BITS);

fn add_unsigned(lw: u32, lf: u32, rw: u32, rf: u32) -> (u32, u32) {
    let mut new_fraction = lf + rf;
    let mut overflow = 0;
    if new_fraction >= DENOMINATOR {
        new_fraction -= DENOMINATOR;
        overflow += 1;
    }
    (lw + rw + overflow, new_fraction)
}

fn subtract_unsigned(lw: u32, lf: u32, rw: u32, rf: u32) -> (bool, u32, u32) {
    let new_fraction;
    let mut borrow = 0;
    if lf >= rf {
        new_fraction = lf - rf;
    } else {
        new_fraction = lf + DENOMINATOR - rf;
        borrow = 1;
    }
    let rw_with_borrow = rw + borrow;
    if rw_with_borrow > lw {
        let whole_diff = rw_with_borrow - lw;
        if new_fraction == 0 {
            (true, whole_diff, 0)
        } else {
            (true, whole_diff - 1, DENOMINATOR - new_fraction)
        }
    } else {
        (false, lw - rw_with_borrow, new_fraction)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Fixed(u32);

impl Fixed {
    pub const ZERO: Fixed = Fixed(0);
    pub const EPSILON: Fixed = Fixed(1);

    pub fn new(whole: i32, fraction: u32) -> Self {
        Self::from_parts(whole < 0, whole.abs() as u32, fraction)
    }

    pub fn from_parts(negative: bool, whole: u32, fraction: u32) -> Self {
        assert!(whole < 2u32.pow(WHOLE_BITS));
        assert!(fraction < DENOMINATOR);
        let sign_bit = if negative { 1 } else { 0 };
        Fixed((sign_bit << 31) | ((whole as u32) << FRACTION_BITS) | fraction)
    }

    pub fn from_f32(value: f32) -> Self {
        let whole = value.trunc().abs() as u32;
        let fraction = ((value - value.trunc()).abs() * DENOMINATOR as f32).round() as u32;
        Self::from_parts(value < 0.0, whole, fraction)
    }

    pub fn vector3_from_f32(v: Vector3<f32>) -> Vector3<Fixed> {
        Vector3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn vector3_to_f32(v: Vector3<Fixed>) -> Vector3<f32> {
        Vector3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn point3_from_f32(v: Point3<f32>) -> Point3<Fixed> {
        Point3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn point3_to_f32(v: Point3<Fixed>) -> Point3<f32> {
        Point3::new(v.x.into(), v.y.into(), v.z.into())
    }

    pub fn unpack(&self) -> (bool, u32, u32) {
        let negative = (0x80000000 & self.0) > 0;
        let whole = (0x7fffff00 & self.0) >> FRACTION_BITS;
        let fraction = 0xff & self.0;
        (negative, whole, fraction)
    }

    pub fn to_f32(&self) -> f32 {
        let (negative, whole, fraction) = self.unpack();
        (whole as f32 + (fraction as f32 / DENOMINATOR as f32)) * if negative { -1.0 } else { 1.0 }
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (negative, whole, fraction) = self.unpack();
        write!(f, "Fixed({}{}.{:03})", if negative { "-" } else { "+" }, whole, fraction)
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
        let (lneg, lw, lf) = self.unpack();
        let (rneg, rw, rf) = rhs.unpack();
        if lneg == rneg {
            let (w, f) = add_unsigned(lw, lf, rw, rf);
            Fixed::from_parts(lneg, w, f)
        } else {
            let (neg, w, f) = subtract_unsigned(lw, lf, rw, rf);
            Fixed::from_parts(neg == rneg, w, f)
        }
    }
}

impl ops::AddAssign<Fixed> for Fixed {
    fn add_assign(&mut self, rhs: Fixed) {
        *self = *self + rhs;
    }
}

impl ops::Sub<Fixed> for Fixed {
    type Output = Fixed;

    fn sub(self, rhs: Fixed) -> Fixed {
        self + (-rhs)
    }
}

impl ops::SubAssign<Fixed> for Fixed {
    fn sub_assign(&mut self, rhs: Fixed) {
        *self = *self - rhs;
    }
}

impl ops::Neg for Fixed {
    type Output = Fixed;

    fn neg(self) -> Fixed {
        if self.0 == 0 {
            return self;
        }
        Fixed(self.0 ^ 0x80000000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_unpack() {
        assert_eq!((false, 128, 0), Fixed::new(128, 0).unpack());
        assert_eq!((false, 2, 4), Fixed::new(2, 4).unpack());
        assert_eq!((true, 6, 5), Fixed::new(-6, 5).unpack());
    }

    #[test]
    fn to_f32() {
        assert_eq!(4.0, Fixed::new(4, 0).to_f32());
        assert_eq!(2.5, Fixed::new(2, 128).to_f32());
    }

    #[test]
    fn from_f32() {
        assert_eq!((false, 2, 0), Fixed::from_f32(2.0).unpack());
        assert_eq!((false, 4, 128), Fixed::from_f32(4.5).unpack());
        assert_eq!((false, 4, 26), Fixed::from_f32(4.1).unpack());
        assert_eq!((true, 0, 128), Fixed::from_f32(-0.5).unpack());
    }

    #[test]
    fn add() {
        assert_eq!(Fixed::new(3, 0), Fixed::new(1, 0) + Fixed::new(2, 0));
        assert_eq!(Fixed::new(100, 10), Fixed::new(99, 0) + Fixed::new(1, 10));
        assert_eq!(Fixed::new(4, 4), Fixed::new(2, 2) + Fixed::new(2, 2));
        assert_eq!(Fixed::new(6, 0), Fixed::new(2, 128) + Fixed::new(3, 128));
        assert_eq!(Fixed::new(10, 11), Fixed::new(2, 255) + Fixed::new(7, 12));
        assert_eq!(Fixed::new(2, 0), Fixed::new(3, 0) + Fixed::new(-1, 0));
        assert_eq!(Fixed::new(-4, 12), Fixed::new(-3, 10) + Fixed::new(-1, 2));
        assert_eq!(Fixed::new(0, 254), Fixed::new(-3, 10) + Fixed::new(4, 8));
        assert_eq!(Fixed::from_parts(true, 0, 246), Fixed::new(-2, 1) + Fixed::new(1, 11));
        assert_eq!(Fixed::new(0, 10), Fixed::new(-2, 1) + Fixed::new(2, 11));
        assert_eq!(Fixed::ZERO, Fixed::new(12, 20) + Fixed::new(-12, 20));
    }

    #[test]
    fn epsilon() {
        let mut f = Fixed::ZERO;
        for _ in 0..DENOMINATOR {
            f += Fixed::EPSILON;
        }
        assert_eq!((false, 1, 0), f.unpack());
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
    fn cgmath_conversions() {
        let v_fixed = Fixed::vector3_from_f32(Vector3::new(1.5f32, 2.25f32, 3.75f32));
        assert_eq!((false, 1, 128), v_fixed.x.unpack());
        assert_eq!((false, 2, 64), v_fixed.y.unpack());
        assert_eq!((false, 3, 192), v_fixed.z.unpack());

        let v_f32 = Fixed::vector3_to_f32(Vector3::new(Fixed::new(1, 128), Fixed::new(2, 64), Fixed::new(3, 192)));
        assert_eq!(1.5, v_f32.x);
        assert_eq!(2.25, v_f32.y);
        assert_eq!(3.75, v_f32.z);

        let p_fixed = Fixed::point3_from_f32(Point3::new(1.5f32, 2.25f32, 3.75f32));
        assert_eq!((false, 1, 128), p_fixed.x.unpack());
        assert_eq!((false, 2, 64), p_fixed.y.unpack());
        assert_eq!((false, 3, 192), p_fixed.z.unpack());

        let p_f32 = Fixed::point3_to_f32(Point3::new(Fixed::new(1, 128), Fixed::new(2, 64), Fixed::new(3, 192)));
        assert_eq!(1.5, p_f32.x);
        assert_eq!(2.25, p_f32.y);
        assert_eq!(3.75, p_f32.z);
    }

    #[test]
    fn debug_format() {
        assert_eq!("Fixed(+2.128)", format!("{:?}", Fixed::new(2, 128)));
        assert_eq!("Fixed(-6.005)", format!("{:?}", Fixed::new(-6, 5)));
        assert_eq!("Fixed(+0.000)", format!("{:?}", Fixed::ZERO));
    }

    #[test]
    fn sub() {
        assert_eq!(Fixed::new(1, 0), Fixed::new(3, 0) - Fixed::new(2, 0));
        assert_eq!(Fixed::new(98, 246), Fixed::new(99, 0) - Fixed::new(0, 10));
        assert_eq!(Fixed::new(0, 0), Fixed::new(2, 2) - Fixed::new(2, 2));
        assert_eq!(Fixed::new(-1, 0), Fixed::new(2, 128) - Fixed::new(3, 128));
        assert_eq!(Fixed::new(-4, 13), Fixed::new(2, 255) - Fixed::new(7, 12));
        assert_eq!(Fixed::new(5, 0), Fixed::new(10, 0) - Fixed::new(5, 0));
        assert_eq!(Fixed::new(-5, 0), Fixed::new(5, 0) - Fixed::new(10, 0));
        assert_eq!(Fixed::new(-15, 0), Fixed::new(-5, 0) - Fixed::new(10, 0));
        assert_eq!(Fixed::new(2, 240), Fixed::new(-4, 5) - Fixed::new(-6, 245));
    }

    #[test]
    fn sub_assign() {
        let mut f = Fixed::new(10, 0);
        f -= Fixed::new(3, 0);
        assert_eq!(Fixed::new(7, 0), f);

        let mut f = Fixed::new(5, 128);
        f -= Fixed::new(2, 64);
        assert_eq!(Fixed::new(3, 64), f);
    }

    #[test]
    fn negate() {
        assert_eq!(Fixed::new(-5, 0), -Fixed::new(5, 0));
        assert_eq!(Fixed::new(5, 0), -Fixed::new(-5, 0));
        assert_eq!(Fixed::new(-2, 128), -Fixed::new(2, 128));
        assert_eq!(Fixed::new(0, 0), -Fixed::ZERO);
    }
}
