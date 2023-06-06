use std::ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Sub, SubAssign};

use super::angle::Radians;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C, align(8))]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self::broadcast(0.0);
    pub const ONE: Self = Self::broadcast(1.0);
    pub const NEG_ONE: Self = Self::broadcast(-1.0);
    pub const MIN: Self = Self::broadcast(f32::MIN);
    pub const MAX: Self = Self::broadcast(f32::MAX);
    pub const NAN: Self = Self::broadcast(f32::NAN);
    pub const INF: Self = Self::broadcast(f32::INFINITY);
    pub const NEG_INF: Self = Self::broadcast(f32::NEG_INFINITY);
    pub const X: Self = Self::new(1.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0);
    pub const AXES: [Self; 2] = [Self::X, Self::Y];

    #[inline(always)]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline(always)]
    pub const fn broadcast(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline(always)]
    pub const fn from_array(src: [f32; 2]) -> Self {
        Self::new(src[0], src[1])
    }

    #[inline(always)]
    pub const fn to_array(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline(always)]
    pub const fn from_slice(slice: &[f32]) -> Self {
        Self::new(slice[0], slice[1])
    }

    #[inline(always)]
    pub fn write_slice(self, slice: &mut [f32]) {
        slice[0] = self.x;
        slice[1] = self.y;
    }

    #[inline(always)]
    pub fn dot(self, rhs: Self) -> f32 {
        (self.x.mul(rhs.x)).add(self.y.mul(rhs.y))
    }

    #[inline(always)]
    pub fn angle_between(self, rhs: Self) -> Radians {
        let dp = self.dot(rhs);
        let mag = self.length() * rhs.length();
        Radians::new(f32::acos((dp / mag).clamp(-1.0, 1.0)))
    }

    #[inline(always)]
    pub fn min(self, rhs: Self) -> Self {
        Self {
            x: self.x.min(rhs.x),
            y: self.y.min(rhs.y),
        }
    }

    #[inline(always)]
    pub fn max(self, rhs: Self) -> Self {
        Self {
            x: self.x.max(rhs.x),
            y: self.y.max(rhs.y),
        }
    }

    #[inline(always)]
    pub fn min_element(self) -> f32 {
        self.x.min(self.y)
    }

    #[inline(always)]
    pub fn max_element(self) -> f32 {
        self.x.max(self.y)
    }

    #[inline(always)]
    pub fn length(self) -> f32 {
        f32::sqrt(self.length_squared())
    }

    #[inline(always)]
    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    #[inline(always)]
    pub fn distance_between(self, rhs: Self) -> f32 {
        self.sub(rhs).length()
    }
}

impl Add<Vec2> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x.add(rhs.x), self.y.add(rhs.y))
    }
}

impl AddAssign<Vec2> for Vec2 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Vec2) {
        self.x.add_assign(rhs.x);
        self.y.add_assign(rhs.y);
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x.sub(rhs.x), self.y.sub(rhs.y))
    }
}

impl SubAssign<Vec2> for Vec2 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Vec2) {
        self.x.sub_assign(rhs.x);
        self.y.sub_assign(rhs.y);
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x.mul(rhs.x), self.y.mul(rhs.y))
    }
}

impl MulAssign<Vec2> for Vec2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Vec2) {
        self.x.mul_assign(rhs.x);
        self.y.mul_assign(rhs.y);
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x.mul(rhs), self.y.mul(rhs))
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: f32) {
        self.x.mul_assign(rhs);
        self.y.mul_assign(rhs);
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Vec2) -> Self::Output {
        Self::new(self.x.div(rhs.x), self.y.div(rhs.y))
    }
}

impl DivAssign<Vec2> for Vec2 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Vec2) {
        self.x.div_assign(rhs.x);
        self.y.div_assign(rhs.y);
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: f32) -> Self::Output {
        Self::new(self.x.div(rhs), self.y.div(rhs))
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline(always)]
    fn div_assign(&mut self, rhs: f32) {
        self.x.div_assign(rhs);
        self.y.div_assign(rhs);
    }
}

impl Index<usize> for Vec2 {
    type Output = f32;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 2);
        if index == 0 {
            return &self.x;
        } else {
            return &self.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_construction() {
        let v = Vec2::default();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);

        let v = Vec2::new(1.0, 2.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);

        let v = Vec2::broadcast(3.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 3.0);

        let v = Vec2::from_array([2.0, 3.0]);
        assert_eq!(v.x, 2.0);
        assert_eq!(v.y, 3.0);

        let slice: [f32; 2] = [3.0, 4.0];
        let v = Vec2::from_slice(&slice);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn test_add() {
        let v1 = Vec2::default();
        let v2 = Vec2::default();
        let sum = v1 + v2;
        let expected = Vec2::default();
        assert_eq!(sum, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let sum = v1 + v2;
        let expected = Vec2::new(1.0 + 3.0, 2.0 + 4.0);
        assert_eq!(sum, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        v1 += v2;
        let expected = Vec2::new(1.0 + 3.0, 2.0 + 4.0);
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_sub() {
        let v1 = Vec2::default();
        let v2 = Vec2::default();
        let diff = v1 - v2;
        let expected = Vec2::default();
        assert_eq!(diff, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let diff = v1 - v2;
        let expected = Vec2::new(1.0 - 3.0, 2.0 - 4.0);
        assert_eq!(diff, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        v1 -= v2;
        let expected = Vec2::new(1.0 - 3.0, 2.0 - 4.0);
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_mul() {
        let v1 = Vec2::default();
        let v2 = Vec2::default();
        let prod = v1 * v2;
        let expected = Vec2::default();
        assert_eq!(prod, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let prod = v1 * v2;
        let expected = Vec2::new(1.0 * 3.0, 2.0 * 4.0);
        assert_eq!(prod, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = 3.0;
        let prod = v1 * v2;
        let expected = Vec2::new(1.0 * 3.0, 2.0 * 3.0);
        assert_eq!(prod, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        v1 *= v2;
        let expected = Vec2::new(1.0 * 3.0, 2.0 * 4.0);
        assert_eq!(v1, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = 3.0;
        v1 *= v2;
        let expected = Vec2::new(1.0 * 3.0, 2.0 * 3.0);
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_div() {
        let v1 = Vec2::default();
        let v2 = Vec2::new(1.0, 1.0);
        let quot = v1 / v2;
        let expected = Vec2::default();
        assert_eq!(quot, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let quot = v1 / v2;
        let expected = Vec2::new(1.0 / 3.0, 2.0 / 4.0);
        assert_eq!(quot, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = 3.0;
        let quot = v1 / v2;
        let expected = Vec2::new(1.0 / 3.0, 2.0 / 3.0);
        assert_eq!(quot, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        v1 /= v2;
        let expected = Vec2::new(1.0 / 3.0, 2.0 / 4.0);
        assert_eq!(v1, expected);

        let mut v1 = Vec2::new(1.0, 2.0);
        let v2 = 3.0;
        v1 /= v2;
        let expected = Vec2::new(1.0 / 3.0, 2.0 / 3.0);
        assert_eq!(v1, expected);
    }

    #[test]
    fn test_index() {
        let v = Vec2::default();
        assert_eq!(v[0], 0.0);
        assert_eq!(v[1], 0.0);

        let v = Vec2::new(1.0, 2.0);
        assert_eq!(v[0], 1.0);
        assert_eq!(v[1], 2.0);

        let v = Vec2::broadcast(3.0);
        assert_eq!(v[0], 3.0);
        assert_eq!(v[1], 3.0);
    }

    #[test]
    fn test_length() {
        let v = Vec2::default();
        let len = v.length();
        let len_squared = v.length_squared();
        let len_expected = 0.0;
        let len_squared_expected = 0.0;
        assert_eq!(len, len_expected);
        assert_eq!(len_squared, len_squared_expected);

        let v = Vec2::new(3.0, 4.0);
        let len = v.length();
        let len_squared = v.length_squared();
        let len_expected = f32::sqrt(3.0 * 3.0 + 4.0 * 4.0);
        let len_squared_expected = 3.0 * 3.0 + 4.0 * 4.0;
        assert_eq!(len, len_expected);
        assert_eq!(len_squared, len_squared_expected);

        let v1 = Vec2::new(3.0, 4.0);
        let v2 = Vec2::new(6.0, 8.0);
        let distance = v1.distance_between(v2);
        let expected = f32::sqrt((3.0 - 6.0) * (3.0 - 6.0) + (4.0 - 8.0) * (4.0 - 8.0));
        assert_eq!(distance, expected);
        assert_eq!(v1.distance_between(v2), v2.distance_between(v1)); // commutative
    }

    #[test]
    fn test_dot() {
        let v1 = Vec2::default();
        let v2 = Vec2::default();
        let dp = v1.dot(v2);
        let expected = 0.0;
        assert_eq!(dp, expected);

        let v1 = Vec2::default();
        let v2 = Vec2::new(1.0, 2.0);
        let dp = v1.dot(v2);
        let expected = 0.0 * 1.0 + 0.0 * 2.0; // 0.0
        assert_eq!(dp, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::default();
        let dp = v1.dot(v2);
        let expected = 1.0 * 0.0 + 2.0 * 0.0; // 0.0
        assert_eq!(dp, expected);

        let v1 = Vec2::new(1.0, 2.0);
        let v2 = Vec2::new(3.0, 4.0);
        let dp = v1.dot(v2);
        let expected = 1.0 * 3.0 + 2.0 * 4.0; // 11.0
        assert_eq!(dp, expected);
    }

    #[test]
    fn test_angle_between() {
        let v1 = Vec2::X;
        let v2 = Vec2::X;
        let angle = v1.angle_between(v2);
        let expected_angle = Radians::new(0.0);
        assert_eq!(angle, expected_angle);

        let v1 = Vec2::X;
        let v2 = Vec2::Y;
        let angle = v1.angle_between(v2);
        let expected_angle = Radians::new(PI / 2.0);
        assert_eq!(angle, expected_angle);

        let v1 = Vec2::new(1.0, 1.0);
        let v2 = Vec2::new(-1.0, 1.0);
        let angle = v1.angle_between(v2);
        let expected_angle = Radians::new(PI / 2.0);
        assert_eq!(angle, expected_angle);

        let v1 = Vec2::new(1.0, 1.0);
        let v2 = Vec2::new(-1.0, -1.0);
        let angle = v1.angle_between(v2);
        let expected_angle = Radians::new(PI);
        assert_eq!(angle, expected_angle);
    }

    #[test]
    fn test_bounds() {
        let v1 = Vec2::default();
        let min = v1.min_element();
        let max = v1.max_element();
        let expected_min = f32::default();
        let expected_max = f32::default();
        assert_eq!(min, expected_min);
        assert_eq!(max, expected_max);

        let v1 = Vec2::new(-1.0, 2.0);
        let min = v1.min_element();
        let max = v1.max_element();
        let expected_min = -1.0;
        let expected_max = 2.0;
        assert_eq!(min, expected_min);
        assert_eq!(max, expected_max);

        let v1 = Vec2::default();
        let v2 = Vec2::default();
        let min = v1.min(v2);
        let max = v1.max(v2);
        let expected_min = Vec2::default();
        let expected_max = Vec2::default();
        assert_eq!(min, expected_min);
        assert_eq!(max, expected_max);
        // check for commutative
        assert_eq!(v1.min(v2), v2.min(v1));
        assert_eq!(v1.max(v2), v2.max(v1));

        let v1 = Vec2::new(-1.0, 2.0);
        let v2 = Vec2::new(3.0, -4.0);
        let min = v1.min(v2);
        let max = v1.max(v2);
        let expected_min = Vec2::new(-1.0, -4.0);
        let expected_max = Vec2::new(3.0, 2.0);
        assert_eq!(min, expected_min);
        assert_eq!(max, expected_max);
        // check for commutative
        assert_eq!(v1.min(v2), v2.min(v1));
        assert_eq!(v1.max(v2), v2.max(v1));
    }
}
