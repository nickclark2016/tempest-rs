use std::f32::consts::PI;

pub trait Angle {
    fn normalize(self) -> Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Radians {
    pub value: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Degrees {
    pub value: f32,
}

impl Radians {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl Angle for Radians {
    fn normalize(self) -> Self {
        Self {
            value: self.value % (PI * 2.0),
        }
    }
}

impl From<Degrees> for Radians {
    fn from(value: Degrees) -> Self {
        Radians::new(value.value * PI / 180.0)
    }
}

impl ToString for Radians {
    fn to_string(&self) -> String {
        format!("{} Radians", self.value)
    }
}

impl Degrees {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

impl Angle for Degrees {
    fn normalize(self) -> Self {
        Self {
            value: self.value % 360.0,
        }
    }
}

impl From<Radians> for Degrees {
    fn from(value: Radians) -> Self {
        Degrees::new(value.value / PI * 180.0)
    }
}

impl ToString for Degrees {
    fn to_string(&self) -> String {
        format!("{} Degrees", self.value)
    }
}
