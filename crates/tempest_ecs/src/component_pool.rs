use std::any::Any;

use super::{component::Component, sparse_index::SparseTableIndex, sparse_map::SparseMap};

pub trait ComponentPool<E: SparseTableIndex> {
    fn erase(&mut self, entity: E) -> bool;
    fn contains(&self, entity: E) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<K: SparseTableIndex + 'static, V: Component + 'static, const PAGE_SIZE: usize> ComponentPool<K>
    for SparseMap<K, V, PAGE_SIZE>
{
    fn erase(&mut self, entity: K) -> bool {
        self.remove(entity).is_some()
    }

    fn contains(&self, entity: K) -> bool {
        self.contains(entity)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
