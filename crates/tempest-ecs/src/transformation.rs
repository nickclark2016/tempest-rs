use tempest_math::f32::{mat4::Mat4, vec3::Vec3};

use crate::component::Component;

#[derive(Component)]
pub struct Transformation {
    pub mat: Mat4,
}

impl Transformation {
    pub fn new(_position: Vec3, _euler_rotation: Vec3, _scale: Vec3) -> Self {
        todo!("Implement new transformation")
    }
}