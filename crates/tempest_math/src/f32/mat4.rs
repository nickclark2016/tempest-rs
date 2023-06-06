use super::vec4::Vec4;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C, align(16))]
pub struct Mat4 {
    pub arr: [f32; 16],
}

impl Mat4 {
    pub const fn from_array(src: [f32; 16]) -> Self {
        Self { arr: src }
    }

    pub const fn from_slice(slice: &[f32]) -> Self {
        Self {
            arr: [
                slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
                slice[8], slice[9], slice[10], slice[11], slice[12], slice[13], slice[14],
                slice[15],
            ],
        }
    }

    pub const fn from_cols(col0: Vec4, col1: Vec4, col2: Vec4, col3: Vec4) -> Self {
        Self {
            arr: [
                col0.x, col0.y, col0.z, col0.w, col1.x, col1.y, col1.z, col1.w, col2.x, col2.y,
                col2.z, col2.w, col3.x, col3.y, col3.z, col3.w,
            ],
        }
    }

    pub const fn broadcast(v: f32) -> Self {
        Self { arr: [v; 16] }
    }

    pub const fn diagonal(diag: Vec4) -> Self {
        Self {
            arr: [
                diag.x, 0.0, 0.0, 0.0, 0.0, diag.y, 0.0, 0.0, 0.0, 0.0, diag.z, 0.0, 0.0, 0.0, 0.0,
                diag.w,
            ],
        }
    }

    pub const fn identity() -> Self {
        Self::diagonal(Vec4::ONE)
    }
}
