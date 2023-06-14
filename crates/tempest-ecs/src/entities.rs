#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Entity {
    id: u32,
    generation: u32,
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }

    pub fn index(&self) -> u32 {
        self.id
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn from_raw(value: u32) -> Self {
        Self {
            id: value,
            generation: 0,
        }
    }
}
