use super::registry::Registry;

pub struct World {
    entities: Registry,
}

impl Default for World {
    fn default() -> Self {
        Self {
            entities: Default::default(),
        }
    }
}

impl World {
    pub fn entities(&self) -> &Registry {
        &self.entities
    }

    pub fn entitites_mut(&mut self) -> &mut Registry {
        &mut self.entities
    }
}
