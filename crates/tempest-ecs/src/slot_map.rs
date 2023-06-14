use std::{
    alloc::{self, Layout},
    mem::{needs_drop, size_of},
    ptr::NonNull,
};

#[derive(Clone, Copy, Eq, Debug, PartialEq)]
pub struct SlotMapKey {
    pub index: u32,
    pub generation: u32,
}

pub struct SlotMap<T> {
    jump: Option<NonNull<SlotMapKey>>,
    values: Option<NonNull<T>>,
    erase: Option<NonNull<u32>>,
    len: usize,
    capacity: usize,
    free_list_ends: Option<(u32, u32)>,
}

pub struct SlotMapValues<'a, T> {
    slot_map: &'a SlotMap<T>,
    index: usize,
}

impl SlotMapKey {
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

impl<T> Drop for SlotMap<T> {
    fn drop(&mut self) {
        match self.jump {
            Some(ptr) => {
                unsafe {
                    let jump_layout = Layout::array::<SlotMapKey>(self.capacity).unwrap();
                    // no drop needed
                    alloc::dealloc(ptr.as_ptr() as *mut u8, jump_layout);
                }
            }
            _ => {}
        };

        match self.values {
            Some(ptr) => unsafe {
                let value_layout = Layout::array::<T>(self.capacity).unwrap();
                if needs_drop::<T>() {
                    for i in 0..self.len {
                        ptr.as_ptr().add(i).drop_in_place();
                    }
                }
                alloc::dealloc(ptr.as_ptr() as *mut u8, value_layout);
            },
            _ => {}
        }

        match self.erase {
            Some(ptr) => unsafe {
                let erase_layout = Layout::array::<u32>(self.capacity).unwrap();
                // no drop needed
                alloc::dealloc(ptr.as_ptr() as *mut u8, erase_layout);
            },
            _ => {}
        }
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        SlotMap {
            jump: None,
            values: None,
            erase: None,
            len: 0,
            capacity: 0,
            free_list_ends: None,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    pub fn insert(&mut self, value: T) -> SlotMapKey {
        if self.is_full() {
            self.capacity = self.grow_allocation(self.capacity + 1);
        }

        let (head, tail) = self.free_list_ends.unwrap();
        let next = unsafe {
            self.jump
                .unwrap_unchecked()
                .as_ptr()
                .add(head as usize)
                .as_ref()
                .unwrap()
                .index
        };

        self.free_list_ends = Some((next, tail));
        let trampoline = unsafe {
            self.jump
                .unwrap_unchecked()
                .as_ptr()
                .add(head as usize)
                .as_mut()
        }
        .unwrap();
        trampoline.index = self.len() as u32;

        unsafe {
            self.values
                .unwrap_unchecked()
                .as_ptr()
                .add(self.len())
                .write(value);
            self.erase
                .unwrap_unchecked()
                .as_ptr()
                .add(trampoline.index as usize)
                .write(head);
        };

        self.len += 1;

        SlotMapKey {
            index: head,
            generation: trampoline.generation,
        }
    }

    pub fn get(&self, key: SlotMapKey) -> Option<&T> {
        if key.index as usize >= self.capacity() {
            return None;
        }

        let trampoline = unsafe {
            self.jump
                .unwrap_unchecked()
                .as_ptr()
                .add(key.index as usize)
                .as_ref()
                .unwrap()
        };
        if trampoline.generation == key.generation {
            unsafe {
                self.values
                    .unwrap_unchecked()
                    .as_ptr()
                    .add(trampoline.index as usize)
                    .as_ref()
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
        if key.index as usize >= self.capacity() {
            return None;
        }

        let trampoline = unsafe {
            self.jump
                .unwrap_unchecked()
                .as_ptr()
                .add(key.index as usize)
                .as_ref()
                .unwrap()
        };
        if trampoline.generation == key.generation {
            unsafe {
                self.values
                    .unwrap_unchecked()
                    .as_ptr()
                    .add(trampoline.index as usize)
                    .as_mut()
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: SlotMapKey) -> Option<T> {
        if key.index as usize >= self.capacity() {
            return None;
        }

        let trampoline = unsafe {
            self.jump
                .unwrap_unchecked()
                .as_ptr()
                .add(key.index as usize)
                .as_ref()
                .unwrap()
        };
        if trampoline.generation == key.generation {
            let idx_to_erase = trampoline.index as usize;
            let value_removed = if idx_to_erase != self.len() - 1 {
                unsafe {
                    let back_erase = self
                        .erase
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(self.len() - 1)
                        .as_ref()
                        .unwrap();
                    self.jump
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(*back_erase as usize)
                        .as_mut()
                        .unwrap()
                        .index = idx_to_erase as u32;

                    let value_removed = self
                        .values
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(idx_to_erase)
                        .read();

                    if needs_drop::<T>() {
                        self.values
                            .unwrap_unchecked()
                            .as_ptr()
                            .add(idx_to_erase)
                            .drop_in_place();
                    }

                    self.values
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(idx_to_erase)
                        .write(
                            self.values
                                .unwrap_unchecked()
                                .as_ptr()
                                .add(self.len() - 1)
                                .read(),
                        );

                    value_removed
                }
            } else {
                unsafe {
                    let value = self
                        .values
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(self.len() - 1)
                        .read();
                    self.values
                        .unwrap_unchecked()
                        .as_ptr()
                        .add(self.len() - 1)
                        .drop_in_place();
                    value
                }
            };

            let next = self.free_list_ends.unwrap().0;
            unsafe {
                let freed = self
                    .jump
                    .unwrap_unchecked()
                    .as_ptr()
                    .add(key.index as usize)
                    .as_mut()
                    .unwrap();
                freed.index = next;
                freed.generation += 1;
            }

            self.free_list_ends.as_mut().unwrap().0 = key.index;
            self.len -= 1;

            Some(value_removed)
        } else {
            None
        }
    }

    pub fn at_index(&self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }

        match self.values {
            Some(values) => unsafe { Some(values.as_ptr().add(index).read()) },
            None => None,
        }
    }

    pub fn at_index_ref(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }

        match self.values {
            Some(values) => unsafe { values.as_ptr().add(index).as_ref() },
            None => None,
        }
    }

    pub fn at_index_mut(&self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }

        match self.values {
            Some(values) => unsafe { values.as_ptr().add(index).as_mut() },
            None => None,
        }
    }

    fn grow_allocation(&mut self, requested_size: usize) -> usize {
        if requested_size < self.capacity {
            return self.capacity;
        }

        let aligned_request = Self::bit_ceil(requested_size);

        let new_jump_ptr = if self.is_empty() {
            unsafe {
                let new_jump_layout = Layout::array::<SlotMapKey>(aligned_request).unwrap();
                let new_ptr = alloc::alloc(new_jump_layout) as *mut SlotMapKey;

                for i in self.len()..aligned_request {
                    new_ptr.add(i).write(SlotMapKey {
                        index: (i + 1) as u32,
                        generation: 0,
                    });
                }

                self.free_list_ends = Some((self.len() as u32, aligned_request as u32 - 1));

                new_ptr
            }
        } else {
            let old_layout = Layout::array::<SlotMapKey>(self.capacity).unwrap();

            unsafe {
                let old_ptr = self.jump.unwrap_unchecked().as_ptr() as *mut u8;
                let new_ptr =
                    alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<SlotMapKey>()) as *mut SlotMapKey;

                for i in self.len()..aligned_request {
                    new_ptr.add(i).write(SlotMapKey {
                        index: (i + 1) as u32,
                        generation: 0,
                    });
                }

                new_ptr
            }
        };

        self.jump = NonNull::new(new_jump_ptr);

        let new_value_ptr = if self.is_empty() {
            let new_value_layout = Layout::array::<T>(aligned_request).unwrap();
            unsafe { alloc::alloc(new_value_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();

            unsafe {
                let old_ptr = self.values.unwrap_unchecked().as_ptr() as *mut u8;
                alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<T>()) as *mut T
            }
        };
        self.values = NonNull::new(new_value_ptr);

        let new_erase_ptr = if self.is_empty() {
            let new_erase_layout = Layout::array::<u32>(aligned_request).unwrap();
            unsafe { alloc::alloc(new_erase_layout) as *mut u32 }
        } else {
            let old_layout = Layout::array::<u32>(self.capacity).unwrap();

            unsafe {
                let old_ptr = self.erase.unwrap_unchecked().as_ptr() as *mut u8;
                alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<u32>()) as *mut u32
            }
        };

        self.erase = NonNull::new(new_erase_ptr);

        aligned_request
    }

    fn bit_ceil(n: usize) -> usize {
        let mut x = n - 1;
        x |= x >> 1;
        x |= x >> 2;
        x |= x >> 4;
        x |= x >> 8;
        x |= x >> 16;
        if std::mem::size_of::<usize>() == 8 {
            x |= x >> 32;
        }
        x + 1
    }
}

impl<'a, T> Iterator for SlotMapValues<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slot_map.len() {
            let value = unsafe {
                self.slot_map
                    .values
                    .unwrap_unchecked()
                    .as_ptr()
                    .add(self.index)
                    .as_ref()
                    .unwrap()
            };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

impl<T> SlotMap<T> {
    pub fn values(&self) -> SlotMapValues<'_, T> {
        SlotMapValues {
            slot_map: &self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_single() {
        let mut map: SlotMap<u32> = SlotMap::new();
        let key = map.insert(42);

        assert_eq!(map.len(), 1);
        assert_eq!(map.capacity(), 1);
        assert_eq!(map.is_empty(), false);
        assert_eq!(map.is_full(), true);

        let value = map.get(key).unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_insert_multiple() {
        let mut map: SlotMap<u32> = SlotMap::new();

        for i in 0..10 {
            map.insert(i);
        }

        assert_eq!(map.len(), 10);
        assert_eq!(map.capacity(), 16); // capacity should have grown

        for i in 0..10 {
            let key = SlotMapKey {
                index: i as u32,
                generation: 0,
            };
            let value = map.get(key).unwrap();
            assert_eq!(*value, i);
        }
    }

    #[test]
    fn test_remove_empty() {
        let mut map: SlotMap<u32> = SlotMap::new();
        assert_eq!(map.remove(SlotMapKey::new(0, 0)), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut map: SlotMap<u32> = SlotMap::new();
        let key = map.insert(42);
        assert_eq!(
            map.remove(SlotMapKey::new(key.index + 1, key.generation)),
            None
        );
        assert_eq!(map.get(key), Some(&42));
    }

    #[test]
    fn test_remove_existing() {
        let mut map: SlotMap<u32> = SlotMap::new();
        let key = map.insert(42);
        assert_eq!(map.remove(key), Some(42));
        assert_eq!(map.get(key), None);
    }

    #[test]
    fn test_remove_multiple() {
        let mut map: SlotMap<u32> = SlotMap::new();
        let key1 = map.insert(42);
        let key2 = map.insert(43);
        assert_eq!(map.remove(key1), Some(42));
        assert_eq!(map.get(key1), None);
        assert_eq!(map.remove(key2), Some(43));
        assert_eq!(map.get(key2), None);
        assert_eq!(map.remove(key1), None);
        assert_eq!(map.remove(key2), None);
    }

    #[test]
    fn test_keys_are_invalidated_on_remove() {
        let mut map = SlotMap::new();
        let key1 = map.insert(1);
        let key2 = map.insert(2);
        let key3 = map.insert(3);

        map.remove(key2);

        assert!(map.get(key1).is_some());
        assert!(map.get(key2).is_none());
        assert!(map.get(key3).is_some());

        let key4 = map.insert(4);

        assert!(map.get(key1).is_some());
        assert!(map.get(key2).is_none());
        assert!(map.get(key3).is_some());
        assert!(map.get(key4).is_some());
    }

    #[test]
    fn test_slotmap_values_iterator() {
        let mut sm = SlotMap::new();
        let _k1 = sm.insert("hello");
        let k2 = sm.insert("world");
        let k3 = sm.insert("foo");
        let _k4 = sm.insert("bar");

        let values: Vec<&str> = sm.values().map(|v| *v).collect();
        assert_eq!(values, vec!["hello", "world", "foo", "bar"]);

        sm.remove(k2);
        sm.remove(k3);

        let values: Vec<&str> = sm.values().map(|v| *v).collect();
        assert_eq!(values, vec!["hello", "bar"]);
    }

    #[test]
    fn test_for_each_values() {
        let mut map = SlotMap::new();

        let a = map.insert("foo");
        let b = map.insert("bar");
        let _c = map.insert("baz");

        let mut values = Vec::new();
        for value in map.values() {
            values.push(*value);
        }

        assert_eq!(values.len(), 3);
        assert!(values.contains(&"foo"));
        assert!(values.contains(&"bar"));
        assert!(values.contains(&"baz"));

        map.remove(a);
        map.remove(b);

        values.clear();
        for value in map.values() {
            values.push(&value);
        }

        assert_eq!(values.len(), 1);
        assert!(values.contains(&"baz"));
    }
}
