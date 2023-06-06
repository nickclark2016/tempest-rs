use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Sub, SubAssign};

use super::{angle::Radians, vec2::Vec2, vec3::Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C, align(16))]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self::broadcast(0.0);
    pub const ONE: Self = Self::broadcast(1.0);
    pub const NEG_ONE: Self = Self::broadcast(-1.0);
    pub const MIN: Self = Self::broadcast(f32::MIN);
    pub const MAX: Self = Self::broadcast(f32::MAX);
    pub const NAN: Self = Self::broadcast(f32::NAN);
    pub const INF: Self = Self::broadcast(f32::INFINITY);
    pub const NEG_INF: Self = Self::broadcast(f32::NEG_INFINITY);
    pub const X: Self = Self::new(1.0, 0.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0, 0.0);
    pub const W: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const AXES: [Self; 4] = [Self::X, Self::Y, Self::Z, Self::W];

    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline(always)]
    pub const fn broadcast(v: f32) -> Self {
        Self {
            x: v,
            y: v,
            z: v,
            w: v,
        }
    }

    #[inline(always)]
    pub const fn from_array(src: [f32; 4]) -> Self {
        Self::new(src[0], src[1], src[2], src[3])
    }

    #[inline(always)]
    pub const fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.z, self.w]
    }

    #[inline(always)]
    pub const fn from_slice(slice: &[f32]) -> Self {
        Self::new(slice[0], slice[1], slice[2], slice[3])
    }

    #[inline(always)]
    pub fn write_slice(self, slice: &mut [f32]) {
        slice[0] = self.x;
        slice[1] = self.y;
        slice[2] = self.z;
        slice[3] = self.w;
    }

    #[inline(always)]
    pub fn dot(self, rhs: Self) -> f32 {
        (self.x.mul(rhs.x))
            .add(self.y.mul(rhs.y))
            .add(self.z.mul(rhs.z))
            .add(self.w.mul(rhs.w))
    }

    #[inline(always)]
    pub fn angle_between(self, rhs: Self) -> Radians {
        let dp = self.dot(rhs);
        let mag = self.length() * rhs.length();
        Radians::new(f32::acos((dp.div(mag)).clamp(-1.0, 1.0)))
    }

    #[inline(always)]
    pub fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
            z: self.z.min(rhs.z),
            w: self.w.min(rhs.w),
        }
    }

    #[inline(always)]
    pub fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
            z: self.z.max(rhs.z),
            w: self.w.max(rhs.w),
        }
    }

    #[inline(always)]
    pub fn min_element(self) -> f32 {
        self.x.min(self.y).min(self.z).min(self.w)
    }

    #[inline(always)]
    pub fn max_element(self) -> f32 {
        self.x.max(self.y).max(self.z).max(self.w)
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        f32::sqrt(self.length_squared())
    }

    #[inline(always)]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }
}

impl From<Vec2> for Vec4 {
    #[inline(always)]
    fn from(value: Vec2) -> Self {
        Vec4::new(value.x, value.y, 0.0, 0.0)
    }
}

impl From<(Vec2, f32, f32)> for Vec4 {
    #[inline(always)]
    fn from(value: (Vec2, f32, f32)) -> Self {
        Self::new(value.0.x, value.0.y, value.1, value.2)
    }
}

impl From<Vec3> for Vec4 {
    #[inline(always)]
    fn from(value: Vec3) -> Self {
        Vec4::new(value.x, value.y, value.z, 0.0)
    }
}

impl From<(Vec3, f32)> for Vec4 {
    #[inline(always)]
    fn from(value: (Vec3, f32)) -> Self {
        Vec4::new(value.0.x, value.0.y, value.0.z, value.1)
    }
}

impl Add<Vec4> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Vec4) -> Self::Output {
        Self::new(
            self.x.add(rhs.x),
            self.y.add(rhs.y),
            self.z.add(rhs.z),
            self.w.add(rhs.w),
        )
    }
}

impl AddAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vec4) {
        self.x.add_assign(rhs.x);
        self.y.add_assign(rhs.y);
        self.z.add_assign(rhs.z);
        self.w.add_assign(rhs.w);
    }
}

impl Sub<Vec4> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Vec4) -> Self::Output {
        Self::new(
            self.x.sub(rhs.x),
            self.y.sub(rhs.y),
            self.z.sub(rhs.z),
            self.w.sub(rhs.w),
        )
    }
}

impl SubAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vec4) {
        self.x.sub_assign(rhs.x);
        self.y.sub_assign(rhs.y);
        self.z.sub_assign(rhs.z);
        self.w.sub_assign(rhs.w);
    }
}

impl Mul<Vec4> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Vec4) -> Self::Output {
        Self::new(
            self.x.mul(rhs.x),
            self.y.mul(rhs.y),
            self.z.mul(rhs.z),
            self.w.mul(rhs.w),
        )
    }
}

impl MulAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Vec4) {
        self.x.mul_assign(rhs.x);
        self.y.mul_assign(rhs.y);
        self.z.mul_assign(rhs.z);
        self.w.mul_assign(rhs.w);
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(
            self.x.mul(rhs),
            self.y.mul(rhs),
            self.z.mul(rhs),
            self.w.mul(rhs),
        )
    }
}

impl MulAssign<f32> for Vec4 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
        self.z.mul_assign(rhs);
        self.w.mul_assign(rhs);
    }
}

impl Div<Vec4> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Vec4) -> Self::Output {
        Self::new(
            self.x.div(rhs.x),
            self.y.div(rhs.y),
            self.z.div(rhs.z),
            self.w.div(rhs.w),
        )
    }
}

impl DivAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Vec4) {
        self.x.div_assign(rhs.x);
        self.y.div_assign(rhs.y);
        self.z.div_assign(rhs.z);
        self.w.div_assign(rhs.w);
    }
}

impl Div<f32> for Vec4 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(
            self.x.div(rhs),
            self.y.div(rhs),
            self.z.div(rhs),
            self.w.div(rhs),
        )
    }
}

impl DivAssign<f32> for Vec4 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f32) {
        self.x.div_assign(rhs);
        self.y.div_assign(rhs);
        self.z.div_assign(rhs);
        self.w.div_assign(rhs);
    }
}

impl Index<usize> for Vec4 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4);
        if index == 0 {
            &self.x
        } else if index == 1 {
            &self.y
        } else if index == 2 {
            &self.z
        } else {
            &self.w
        }
    }
}
