use std::marker::PhantomData;

pub use tempest_ecs_macros::RegistryQuery;

use super::{
    component::Component,
    component_pool::ComponentPool,
    slot_map::{SlotMap, SlotMapKey},
    sparse_index::SparseTableIndex,
    sparse_map::SparseMap,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct EntityKey {
    pub(crate) id: usize,
}

impl SparseTableIndex for EntityKey {
    fn index(self) -> u32 {
        self.id as u32
    }

    fn tombstone() -> Self {
        Self { id: !0 }
    }

    fn from_raw(value: u32) -> Self {
        Self { id: value as usize }
    }
}

#[derive(Default)]
pub struct Registry {
    pools: Vec<Option<Box<dyn ComponentPool<EntityKey>>>>,
    entities: SlotMap<EntityKey>,
}

#[derive(Clone, Copy)]
pub struct Entity {
    pub id: SlotMapKey,
}

impl Registry {
    pub fn create_entity(&mut self) -> Entity {
        let entity_id = self.entities.len();
        let ent_key = EntityKey { id: entity_id };
        let key = self.entities.insert(ent_key);

        Entity { id: key }
    }

    pub fn destroy_entity(&mut self, ent: &Entity) -> bool {
        let key = ent.id;
        let ent_key = self.entities.get(key);

        if let Some(k) = ent_key {
            self.pools.iter_mut().for_each(|pool| {
                if let Some(p) = pool {
                    p.erase(*k);
                }
            });

            self.entities.remove(key);

            return true;
        }

        false
    }

    pub fn assign_component<T: Component>(&mut self, ent: Entity, component: T) -> bool {
        let id = self.entities.get(ent.id).map(|k| *k);
        let pool = self.fetch_or_create_pool::<T>();

        match id {
            Some(id) => {
                pool.insert(id.clone(), component);
                true
            }
            None => false,
        }
    }

    pub fn get_component<T: Component>(&self, ent: Entity) -> Option<T> {
        let id = self.entities.get(ent.id);
        let pool = self.fetch_pool::<T>();

        if let Some(id) = id {
            if let Some(pool) = pool {
                return pool.get(id.clone()).map(|v| *v);
            }
        }

        None
    }

    pub fn has_component<T: Component>(&self, ent: Entity) -> bool {
        let id = self.entities.get(ent.id);
        let pool = self.fetch_pool::<T>();

        if let Some(id) = id {
            if let Some(pool) = pool {
                return pool.contains(*id);
            }
        }

        false
    }

    pub fn has_component_id(&self, ent: Entity, component_id: usize) -> bool {
        let id = self.entities.get(ent.id);
        if let Some(id) = id {
            let pool = self.fetch_pool_base(component_id);
            if let Some(pool) = pool {
                return pool.contains(*id);
            }
        }

        false
    }

    pub fn remove_component<T: Component>(&mut self, ent: Entity) -> Option<T> {
        let id = self.entities.get(ent.id).map(|k| *k);
        let pool = self.fetch_pool_mut::<T>();
        match id {
            Some(id) => match pool {
                Some(p) => p.remove(id),
                None => None,
            },
            None => None,
        }
    }

    pub fn num_entities(&self) -> usize {
        self.entities.len()
    }

    pub fn entity_capacity(&self) -> usize {
        self.entities.capacity()
    }

    fn fetch_or_create_pool<T: Component>(&mut self) -> &mut SparseMap<EntityKey, T, 1024> {
        let id = T::id();

        if id < self.pools.len() {
            let pool = &self.pools[id];
            match pool {
                Some(_) => {}
                None => return self.register_pool::<T>(),
            }
        } else {
            return self.register_pool::<T>();
        };

        unsafe {
            self.pools[id]
                .as_mut()
                .unwrap_unchecked()
                .as_any_mut()
                .downcast_mut::<SparseMap<EntityKey, T, 1024>>()
                .unwrap_unchecked()
        }
    }

    pub(crate) fn fetch_pool<T: Component>(&self) -> Option<&SparseMap<EntityKey, T, 1024>> {
        let id = T::id();

        if id < self.pools.len() {
            let pool = &self.pools[id];
            match pool {
                Some(_) => {}
                None => return None,
            }
        } else {
            return None;
        };

        unsafe {
            self.pools[id]
                .as_ref()
                .unwrap_unchecked()
                .as_any()
                .downcast_ref::<SparseMap<EntityKey, T, 1024>>()
        }
    }

    fn fetch_pool_base(&self, id: usize) -> Option<&Box<dyn ComponentPool<EntityKey>>> {
        if id < self.pools.len() {
            let pool = &self.pools[id];
            match pool {
                Some(_) => {}
                None => return None,
            }
        } else {
            return None;
        };

        unsafe { Some(self.pools[id].as_ref().unwrap_unchecked()) }
    }

    fn fetch_pool_mut<T: Component>(&mut self) -> Option<&mut SparseMap<EntityKey, T, 1024>> {
        let id = T::id();

        if id < self.pools.len() {
            let pool = &self.pools[id];
            match pool {
                Some(_) => {}
                None => return None,
            }
        } else {
            return None;
        };

        unsafe {
            self.pools[id]
                .as_mut()
                .unwrap_unchecked()
                .as_any_mut()
                .downcast_mut::<SparseMap<EntityKey, T, 1024>>()
        }
    }

    fn register_pool<T: Component>(&mut self) -> &mut SparseMap<EntityKey, T, 1024> {
        let id: usize = T::id();
        assert!(id >= self.pools.len());

        self.pools.resize_with(id + 1, || None);

        let pool = SparseMap::<EntityKey, T, 1024>::default();
        self.pools[id] = Some(Box::new(pool));

        unsafe {
            self.pools[id]
                .as_mut()
                .unwrap_unchecked()
                .as_any_mut()
                .downcast_mut::<SparseMap<EntityKey, T, 1024>>()
                .unwrap_unchecked()
        }
    }
}

#[derive(Clone, Copy)]
pub struct QueryIterator {
    pub id: usize,
}

impl Registry {
    pub fn contains_component_from_iter<T: Component>(&self, it: QueryIterator) -> bool {
        let entity = self.entities.at_index(it.id);
        if let Some(entity) = entity {
            let pool = self.fetch_pool::<T>();
            if let Some(pool) = pool {
                pool.contains(entity)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_component_from_iter<T: Component>(&self, it: QueryIterator) -> Option<T> {
        let entity = self.entities.at_index(it.id);
        if let Some(entity) = entity {
            let pool = self.fetch_pool::<T>();
            if let Some(pool) = pool {
                pool.get(entity).map(|v| *v)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_component_ref_from_iter<T: Component>(&self, it: QueryIterator) -> Option<&T> {
        let entity = self.entities.at_index(it.id);
        if let Some(entity) = entity {
            let pool = self.fetch_pool::<T>();
            if let Some(pool) = pool {
                pool.get(entity)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_component_mut_from_iter<T: Component>(&self, it: QueryIterator) -> Option<&mut T> {
        let entity = self.entities.at_index(it.id);
        if let Some(entity) = entity {
            let pool = self.fetch_pool::<T>();
            if let Some(pool) = pool {
                pool.get_mut(entity)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait RegistryQuery<'r> {
    type Result;
    fn contains(it: QueryIterator, reg: &'r Registry) -> bool;
    fn fetch(it: QueryIterator, reg: &'r Registry) -> Option<Self::Result>;
}

pub struct RegistryRefQuery<'a, T: RegistryQuery<'a>> {
    reg: &'a Registry,
    index: QueryIterator,
    type_phantom: PhantomData<T>,
}

impl<'a, T: RegistryQuery<'a>> Iterator for RegistryRefQuery<'a, T> {
    type Item = T::Result;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index.id < self.reg.num_entities() {
            let result = match T::fetch(self.index, self.reg) {
                Some(res) => Some(res),
                None => None,
            };

            self.index = QueryIterator {
                id: self.index.id + 1,
            };

            if result.is_some() {
                return result;
            }
        }

        None
    }
}

impl Registry {
    pub fn query_registry<'a, T: RegistryQuery<'a>>(&'a self) -> RegistryRefQuery<'a, T> {
        let init = QueryIterator { id: 0 };
        return RegistryRefQuery {
            reg: &self,
            index: init,
            type_phantom: PhantomData::default(),
        };
    }
}

#[cfg(test)]
mod tests {
    use tempest_ecs_macros::{Component, RegistryQuery};

    use super::*;

    #[derive(Component, Default)]
    struct TestSuiteComponent(u32);

    impl TestSuiteComponent {
        pub fn new(value: u32) -> Self {
            let mut me = Self::default();
            me.0 = value;
            me
        }
    }

    #[derive(Component, Default)]
    struct TestSuiteComponent2(u32);

    #[test]
    fn test_default() {
        let reg = Registry::default();
        assert_eq!(reg.num_entities(), 0);
        assert_eq!(reg.entity_capacity(), 0);
    }

    #[test]
    fn test_create_single_entity() {
        let mut reg = Registry::default();
        let ent = reg.create_entity();

        assert_eq!(reg.num_entities(), 1);
        assert!(reg.entity_capacity() >= reg.num_entities());

        assert!(reg.destroy_entity(&ent));

        assert_eq!(reg.num_entities(), 0);
        assert!(reg.entity_capacity() >= reg.num_entities());
    }

    #[test]
    fn test_create_destroy_create() {
        let mut reg = Registry::default();
        let ent = reg.create_entity();

        assert_eq!(reg.num_entities(), 1);
        assert!(reg.entity_capacity() >= reg.num_entities());

        assert!(reg.destroy_entity(&ent));

        assert_eq!(reg.num_entities(), 0);
        assert!(reg.entity_capacity() >= reg.num_entities());

        let ent2 = reg.create_entity();
        assert_eq!(reg.num_entities(), 1);
        assert!(reg.entity_capacity() >= reg.num_entities());

        assert_ne!(ent.id, ent2.id);
        assert_eq!(ent.id.index, ent2.id.index);
        assert_ne!(ent.id.generation, ent2.id.generation);
    }

    #[test]
    fn test_assign_component() {
        let mut reg = Registry::default();
        let ent = reg.create_entity();

        reg.assign_component(ent, TestSuiteComponent::default());
        assert!(reg.has_component::<TestSuiteComponent>(ent));

        assert!(reg.remove_component::<TestSuiteComponent>(ent).is_some());
        assert!(!reg.has_component::<TestSuiteComponent>(ent));

        reg.destroy_entity(&ent);

        // stick an entity in the same slot, but make sure that the registry doesn't try to assign the component to the old entity
        let ent2 = reg.create_entity();

        assert!(!reg.assign_component(ent, TestSuiteComponent::default()));

        // make sure it can assign it to the new entity
        assert!(reg.assign_component(ent2, TestSuiteComponent::default()));
        assert!(reg.has_component::<TestSuiteComponent>(ent2));
        assert!(reg.remove_component::<TestSuiteComponent>(ent2).is_some());
        assert!(!reg.has_component::<TestSuiteComponent>(ent2));
    }

    #[derive(RegistryQuery)]
    #[read_only(TestSuiteComponent, TestSuiteComponent2)]
    struct MyTestQuery;

    #[test]
    fn test_multi_component_query() {
        let mut reg = Registry::default();
        let mut count = 0;
        for _ in reg.query_registry::<MyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        let entity = reg.create_entity();
        reg.assign_component(entity, TestSuiteComponent::new(1));

        for _ in reg.query_registry::<MyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        reg.assign_component(entity, TestSuiteComponent2::default());

        for (comp0, comp1) in reg.query_registry::<MyTestQuery>() {
            assert_eq!(comp0.0, 1);
            assert_eq!(comp1.0, 0);
            count += 1;
        }

        assert_eq!(count, 1);
        count = 0;

        reg.destroy_entity(&entity);

        for _ in reg.query_registry::<MyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);
    }

    #[derive(RegistryQuery)]
    #[read_write(TestSuiteComponent, TestSuiteComponent2)]
    struct MutMyTestQuery;

    #[test]
    fn test_multi_rw_component_query() {
        let mut reg = Registry::default();
        let mut count = 0;
        for _ in reg.query_registry::<MutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        let entity = reg.create_entity();
        reg.assign_component(entity, TestSuiteComponent::new(1));

        for _ in reg.query_registry::<MutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        reg.assign_component(entity, TestSuiteComponent2::default());

        for (comp0, comp1) in reg.query_registry::<MutMyTestQuery>() {
            assert_eq!(comp0.0, 1);
            assert_eq!(comp1.0, 0);
            count += 1;
        }

        assert_eq!(count, 1);
        count = 0;

        reg.destroy_entity(&entity);

        for _ in reg.query_registry::<MutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);
    }

    #[derive(RegistryQuery)]
    #[read_only(TestSuiteComponent)]
    #[read_write(TestSuiteComponent2)]
    struct MixedMutMyTestQuery;

    #[test]
    fn test_multi_mixed_rw_component_query() {
        let mut reg = Registry::default();
        let mut count = 0;
        for _ in reg.query_registry::<MixedMutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        let entity = reg.create_entity();
        reg.assign_component(entity, TestSuiteComponent::new(1));

        for _ in reg.query_registry::<MixedMutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);

        reg.assign_component(entity, TestSuiteComponent2::default());

        for (comp0, comp1) in reg.query_registry::<MixedMutMyTestQuery>() {
            assert_eq!(comp0.0, 1);
            assert_eq!(comp1.0, 0);
            count += 1;
        }

        assert_eq!(count, 1);
        count = 0;

        reg.destroy_entity(&entity);

        for _ in reg.query_registry::<MixedMutMyTestQuery>() {
            count += 1;
        }

        assert_eq!(count, 0);
    }
}
