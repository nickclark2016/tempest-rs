use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::{needs_drop, size_of},
    ptr::{self, NonNull},
};

use super::sparse_index::SparseTableIndex;

pub struct SparseMap<K: SparseTableIndex, V, const PAGE_SIZE: usize> {
    packed_keys: NonNull<K>,
    packed_values: NonNull<V>,
    sparse_keys: Vec<Box<[u32; PAGE_SIZE]>>,
    cap: usize,
    len: usize,
    key_marker: PhantomData<K>,
    value_marker: PhantomData<V>,
}

pub struct SparseMapIterator<
    'a,
    K: 'a + SparseTableIndex,
    V: 'a + Copy + Clone,
    const PAGE_SIZE: usize,
> {
    keys: NonNull<K>,
    values: NonNull<V>,
    index: usize,
    len: usize,
    key_marker: PhantomData<&'a K>,
    value_marker: PhantomData<&'a V>,
}

pub struct SparseMapMutIterator<
    'a,
    K: 'a + SparseTableIndex,
    V: 'a + Copy + Clone,
    const PAGE_SIZE: usize,
> {
    keys: NonNull<K>,
    values: NonNull<V>,
    index: usize,
    len: usize,
    key_marker: PhantomData<&'a K>,
    value_marker: PhantomData<&'a V>,
}

pub struct IntoIter<K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> {
    map: SparseMap<K, V, PAGE_SIZE>,
}

impl<K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> Default
    for SparseMap<K, V, PAGE_SIZE>
{
    fn default() -> Self {
        Self {
            packed_keys: NonNull::dangling(),
            packed_values: NonNull::dangling(),
            sparse_keys: Vec::default(),
            cap: 0,
            len: 0,
            key_marker: PhantomData,
            value_marker: PhantomData,
        }
    }
}

impl<K: SparseTableIndex, V, const PAGE_SIZE: usize> Drop for SparseMap<K, V, PAGE_SIZE> {
    fn drop(&mut self) {
        if self.cap > 0 {
            if needs_drop::<K>() {
                for i in 0..self.len {
                    unsafe {
                        self.packed_keys.as_ptr().add(i).drop_in_place();
                    }
                }
            }

            if needs_drop::<V>() {
                for i in 0..self.len {
                    unsafe {
                        self.packed_values.as_ptr().add(i).drop_in_place();
                    }
                }
            }

            let packed_key_layout = Layout::array::<K>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.packed_keys.as_ptr() as *mut u8, packed_key_layout);
            }

            let packed_value_layout = Layout::array::<V>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.packed_values.as_ptr() as *mut u8, packed_value_layout);
            }

            self.sparse_keys.clear();
            self.cap = 0;
            self.len = 0;
        }
    }
}

impl<K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> SparseMap<K, V, PAGE_SIZE> {
    pub fn insert(&mut self, key: K, value: V) {
        if self.len >= self.cap || key.index() as usize >= self.cap {
            self.grow_allocation(Some(key.index() as usize + 1));
        }

        let sparse_page_index = self.get_page(key);
        let sparse_page_offset = self.get_offset(key);

        let page = &mut self.sparse_keys[sparse_page_index];
        let vacant = unsafe { *page.as_ptr().add(sparse_page_offset) == K::tombstone().index() };
        if vacant {
            page[sparse_page_offset] = self.len as u32;
            unsafe {
                self.packed_keys.as_ptr().add(self.len).write(key);
                self.packed_values.as_ptr().add(self.len).write(value);
            }
            self.len += 1;
        }
    }

    pub fn contains(&self, key: K) -> bool {
        let sparse_page_index = self.get_page(key);
        let sparse_page_offset = self.get_offset(key);
        if sparse_page_index < self.sparse_keys.len() {
            unsafe {
                self.sparse_keys[sparse_page_index]
                    .as_ptr()
                    .add(sparse_page_offset)
                    .as_ref()
                    .or(Some(&K::tombstone().index()))
                    .map(|k| !k.eq(&K::tombstone().index()))
                    .unwrap_or(false)
            }
        } else {
            false
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let (sparse_page_index, sparse_page_offset) = (self.get_page(key), self.get_offset(key));

        if sparse_page_index < self.sparse_keys.len() {
            unsafe {
                let trampoline = *self.sparse_keys[sparse_page_index]
                    .as_ptr()
                    .add(sparse_page_offset);

                if trampoline != K::tombstone().index() && {
                    *self.packed_keys.as_ptr().add(trampoline as usize)
                }
                .eq(&key)
                {
                    self.packed_values
                        .as_ptr()
                        .add(trampoline as usize)
                        .as_ref()
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn get_mut(&self, key: K) -> Option<&mut V> {
        let (sparse_page_index, sparse_page_offset) = (self.get_page(key), self.get_offset(key));

        if sparse_page_index < self.sparse_keys.len() {
            unsafe {
                let trampoline = *self.sparse_keys[sparse_page_index]
                    .as_ptr()
                    .add(sparse_page_offset);

                if trampoline != K::tombstone().index() && {
                    *self.packed_keys.as_ptr().add(trampoline as usize)
                }
                .eq(&key)
                {
                    self.packed_values
                        .as_ptr()
                        .add(trampoline as usize)
                        .as_mut()
                } else {
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        let (sparse_page_index, sparse_page_offset) = (self.get_page(key), self.get_offset(key));

        if sparse_page_index < self.sparse_keys.len() {
            let trampoline = self.sparse_keys[sparse_page_index][sparse_page_offset] as usize;
            if trampoline != K::tombstone().index() as usize
                && unsafe { *self.packed_keys.as_ptr().add(trampoline) }.eq(&key)
            {
                self.sparse_keys[sparse_page_index][sparse_page_offset] = K::tombstone().index();

                unsafe {
                    let back_key = *self.packed_keys.as_ptr().add(self.len - 1);
                    let back_value = *self.packed_values.as_ptr().add(self.len - 1);

                    let removed_value = *self.packed_values.as_ptr().add(trampoline);

                    (*self.packed_keys.as_ptr().add(trampoline)) = back_key;
                    (*self.packed_values.as_ptr().add(trampoline)) = back_value;

                    if needs_drop::<K>() {
                        self.packed_keys.as_ptr().add(self.len - 1).drop_in_place();
                    }

                    if needs_drop::<V>() {
                        self.packed_values
                            .as_ptr()
                            .add(self.len - 1)
                            .drop_in_place();
                    }

                    self.len -= 1;

                    if !self.is_empty() {
                        let (last_index, last_offset) =
                            (self.get_page(back_key), self.get_offset(back_key));
                        self.sparse_keys[last_index][last_offset] = trampoline as u32;
                    }

                    Some(removed_value)
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn iter(&self) -> SparseMapIterator<'_, K, V, PAGE_SIZE> {
        SparseMapIterator {
            keys: self.packed_keys,
            values: self.packed_values,
            index: 0,
            len: self.len(),
            key_marker: PhantomData,
            value_marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> SparseMapMutIterator<'_, K, V, PAGE_SIZE> {
        SparseMapMutIterator {
            keys: self.packed_keys,
            values: self.packed_values,
            index: 0,
            len: self.len(),
            key_marker: PhantomData,
            value_marker: PhantomData,
        }
    }

    pub fn as_keys_slice(&self) -> &[K] {
        unsafe {
            ptr::slice_from_raw_parts(self.packed_keys.as_ptr(), self.len())
                .as_ref()
                .unwrap_or(&[])
        }
    }

    pub fn as_keys_slice_mut(&mut self) -> &mut [K] {
        unsafe {
            ptr::slice_from_raw_parts_mut(self.packed_keys.as_ptr(), self.len())
                .as_mut()
                .unwrap_or(&mut [])
        }
    }

    pub fn as_value_slice(&self) -> &[V] {
        unsafe {
            ptr::slice_from_raw_parts(self.packed_values.as_ptr(), self.len())
                .as_ref()
                .unwrap_or(&[])
        }
    }

    pub fn as_value_slice_mut(&mut self) -> &mut [V] {
        unsafe {
            ptr::slice_from_raw_parts_mut(self.packed_values.as_ptr(), self.len())
                .as_mut()
                .unwrap_or(&mut [])
        }
    }

    #[inline]
    pub fn at_index(&self, index: usize) -> Option<V> {
        if index >= self.len {
            return None;
        }

        unsafe { Some(self.packed_values.as_ptr().add(index).read()) }
    }

    #[inline]
    pub fn at_index_ref(&self, index: usize) -> Option<&V> {
        if index >= self.len {
            return None;
        }

        unsafe { self.packed_values.as_ptr().add(index).as_ref() }
    }

    #[inline]
    pub fn at_index_mut(&self, index: usize) -> Option<&mut V> {
        if index >= self.len {
            return None;
        }

        unsafe { self.packed_values.as_ptr().add(index).as_mut() }
    }

    fn grow_allocation(&mut self, requested_size: Option<usize>) {
        let request = requested_size
            .or_else(|| {
                let size = if self.cap == 0 { 1 } else { self.cap * 2 };
                Some(size)
            })
            .unwrap();

        self.grow_sparse_allocation(request);
        self.cap = self.grow_packed_allocation(request);
    }

    fn grow_packed_allocation(&mut self, requested_size: usize) -> usize {
        if requested_size < self.cap {
            return self.cap;
        }

        let aligned_request = {
            let d = requested_size / PAGE_SIZE;
            let r = requested_size % PAGE_SIZE;
            if r > 0 && PAGE_SIZE > 0 {
                d + 1
            } else {
                d
            }
        } * PAGE_SIZE;

        let new_key_layout = Layout::array::<K>(aligned_request).unwrap();

        let new_key_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_key_layout) as *mut K }
        } else {
            let old_layout = Layout::array::<K>(self.cap).unwrap();
            let old_ptr = self.packed_keys.as_ptr() as *mut u8;

            unsafe { alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<K>()) as *mut K }
        };

        self.packed_keys = match NonNull::new(new_key_ptr) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_key_layout),
        };

        let new_value_layout = Layout::array::<V>(aligned_request).unwrap();

        let new_value_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_value_layout) as *mut V }
        } else {
            let old_layout = Layout::array::<K>(self.cap).unwrap();
            let old_ptr = self.packed_values.as_ptr() as *mut u8;

            unsafe { alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<V>()) as *mut V }
        };

        self.packed_values = match NonNull::new(new_value_ptr as *mut V) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_value_layout),
        };

        aligned_request
    }

    fn grow_sparse_allocation(&mut self, requested_size: usize) {
        if requested_size < self.cap {
            return;
        }

        let requested_page_count = {
            let d = requested_size / PAGE_SIZE;
            let r = requested_size % PAGE_SIZE;
            if r > 0 && PAGE_SIZE > 0 {
                d + 1
            } else {
                d
            }
        };

        for _ in self.sparse_keys.len()..requested_page_count {
            let page_ptr = Box::new([K::tombstone().index(); PAGE_SIZE]);
            self.sparse_keys.push(page_ptr);
        }
    }

    fn get_page(&self, value: K) -> usize {
        self.get_page_raw(value.index() as usize)
    }

    fn get_page_raw(&self, value: usize) -> usize {
        value / PAGE_SIZE
    }

    fn get_offset(&self, value: K) -> usize {
        self.get_offset_raw(value.index() as usize)
    }

    fn get_offset_raw(&self, value: usize) -> usize {
        value % PAGE_SIZE
    }
}

impl<K: SparseTableIndex, V: Clone + Copy + PartialEq, const PAGE_SIZE: usize>
    SparseMap<K, V, PAGE_SIZE>
{
    pub fn contains_pair(&self, key: K, value: &V) -> bool {
        let sparse_page_index = self.get_page(key);
        let sparse_page_offset = self.get_offset(key);
        if sparse_page_index < self.sparse_keys.len() {
            unsafe {
                let trampoline = *self.sparse_keys[sparse_page_index]
                    .as_ptr()
                    .add(sparse_page_offset);

                if trampoline != K::tombstone().index() && {
                    *self.packed_keys.as_ptr().add(trampoline as usize)
                }
                .eq(&key)
                {
                    { *self.packed_values.as_ptr().add(trampoline as usize) }.eq(&value)
                } else {
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn remove_pair(&mut self, key: K, value: &V) -> bool {
        let (sparse_page_index, sparse_page_offset) = (self.get_page(key), self.get_offset(key));

        if sparse_page_index < self.sparse_keys.len() {
            let trampoline = self.sparse_keys[sparse_page_index][sparse_page_offset] as usize;
            if trampoline != K::tombstone().index() as usize
                && unsafe { *self.packed_keys.as_ptr().add(trampoline) }.eq(&key)
            {
                self.sparse_keys[sparse_page_index][sparse_page_offset] = K::tombstone().index();

                unsafe {
                    let back_key = *self.packed_keys.as_ptr().add(self.len - 1);
                    let back_value = *self.packed_values.as_ptr().add(self.len - 1);

                    let removed_value = *self.packed_values.as_ptr().add(trampoline);
                    if !removed_value.eq(value) {
                        false
                    } else {
                        (*self.packed_keys.as_ptr().add(trampoline)) = back_key;
                        (*self.packed_values.as_ptr().add(trampoline)) = back_value;

                        if needs_drop::<K>() {
                            self.packed_keys.as_ptr().add(self.len - 1).drop_in_place();
                        }

                        if needs_drop::<V>() {
                            self.packed_values
                                .as_ptr()
                                .add(self.len - 1)
                                .drop_in_place();
                        }

                        self.len -= 1;

                        if !self.is_empty() {
                            let (last_index, last_offset) =
                                (self.get_page(back_key), self.get_offset(back_key));
                            self.sparse_keys[last_index][last_offset] = trampoline as u32;
                        }

                        true
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<'a, K: 'a + SparseTableIndex, V: 'a + Copy + Clone, const PAGE_SIZE: usize> Iterator
    for SparseMapIterator<'a, K, V, PAGE_SIZE>
{
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len {
            None
        } else {
            let (key, value) = unsafe {
                #[cfg(debug_assertions)]
                {
                    let key = self
                        .keys
                        .as_ptr()
                        .add(self.index)
                        .as_ref()
                        .expect("Keys pointer was null.");

                    let value = self
                        .values
                        .as_ptr()
                        .add(self.index)
                        .as_ref()
                        .expect("Values pointer was null.");

                    (key, value)
                }
                #[cfg(not(debug_assertions))]
                {
                    let key = self
                        .keys
                        .as_ptr()
                        .add(self.index)
                        .as_ref()
                        .unwrap_unchecked();

                    let value = self
                        .values
                        .as_ptr()
                        .add(self.index)
                        .as_ref()
                        .unwrap_unchecked();

                    (key, value)
                }
            };
            self.index += 1;
            Some((key, value))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> Iterator
    for IntoIter<K, V, PAGE_SIZE>
{
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.map.is_empty() {
            None
        } else {
            let first_key = self.map.as_keys_slice()[0];
            let first_value = self.map.as_value_slice()[0];
            self.map.remove(first_key);
            Some((first_key, first_value))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

impl<K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> IntoIterator
    for SparseMap<K, V, PAGE_SIZE>
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, PAGE_SIZE>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { map: self }
    }
}

impl<'a, K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> IntoIterator
    for &'a SparseMap<K, V, PAGE_SIZE>
{
    type Item = (&'a K, &'a V);
    type IntoIter = SparseMapIterator<'a, K, V, PAGE_SIZE>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K: SparseTableIndex, V: Copy + Clone, const PAGE_SIZE: usize> IntoIterator
    for &'a mut SparseMap<K, V, PAGE_SIZE>
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = SparseMapMutIterator<'a, K, V, PAGE_SIZE>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, K: 'a + SparseTableIndex, V: 'a + Copy + Clone, const PAGE_SIZE: usize> Iterator
    for SparseMapMutIterator<'a, K, V, PAGE_SIZE>
{
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.len {
            None
        } else {
            let (key, value) = unsafe {
                #[cfg(debug_assertions)]
                {
                    let key = self
                        .keys
                        .as_ptr()
                        .add(self.index)
                        .as_mut()
                        .expect("Keys pointer was null.");

                    let value = self
                        .values
                        .as_ptr()
                        .add(self.index)
                        .as_mut()
                        .expect("Values pointer was null.");

                    (key, value)
                }
                #[cfg(not(debug_assertions))]
                {
                    let key = self
                        .keys
                        .as_ptr()
                        .add(self.index)
                        .as_mut()
                        .unwrap_unchecked();

                    let value = self
                        .values
                        .as_ptr()
                        .add(self.index)
                        .as_mut()
                        .unwrap_unchecked();

                    (key, value)
                }
            };
            self.index += 1;
            Some((key, value))
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len - self.index, Some(self.len - self.index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct SimpleKey {
        id: u32,
    }

    impl SparseTableIndex for SimpleKey {
        fn index(self) -> u32 {
            self.id
        }

        fn tombstone() -> Self {
            Self { id: u32::MAX }
        }

        fn from_raw(value: u32) -> Self {
            Self { id: value }
        }
    }

    #[test]
    fn test_insert() {
        let mut map = SparseMap::<SimpleKey, i32, 1024>::default();
        let key1 = SimpleKey { id: 1 };
        let key2 = SimpleKey { id: 2 };
        let value1 = 42;
        let value2 = 43;

        map.insert(key1, value1);
        assert_eq!(map.contains(key1), true);
        assert_eq!(map.contains(key2), false);

        map.insert(key2, value2);
        assert_eq!(map.contains(key1), true);
        assert_eq!(map.contains(key2), true);
    }

    #[test]
    fn test_remove() {
        let mut map = SparseMap::<SimpleKey, i32, 1024>::default();
        let key1 = SimpleKey { id: 1 };
        let key2 = SimpleKey { id: 2 };
        let value1 = 42;
        let value2 = 43;

        map.insert(key1, value1);
        map.insert(key2, value2);

        assert_eq!(map.remove(key1), Some(42));
        assert_eq!(map.contains(key1), false);
        assert_eq!(map.remove(key1), None);

        assert_eq!(map.remove(key2), Some(43));
        assert_eq!(map.contains(key2), false);
        assert_eq!(map.remove(key2), None);
    }

    #[test]
    fn test_contains() {
        let mut map = SparseMap::<SimpleKey, i32, 1024>::default();
        let key1 = SimpleKey { id: 1 };
        let key2 = SimpleKey { id: 2 };
        let value1 = 42;

        map.insert(key1, value1);

        assert_eq!(map.contains(key1), true);
        assert_eq!(map.contains(key2), false);
    }
    #[test]
    fn test_iter() {
        let mut map = SparseMap::<SimpleKey, i32, 1024>::default();
        map.insert(SimpleKey { id: 1 }, 10);
        map.insert(SimpleKey { id: 3 }, 30);
        map.insert(SimpleKey { id: 5 }, 50);
        map.insert(SimpleKey { id: 7 }, 70);

        let mut iter = map.iter();
        assert_eq!(iter.next(), Some((&SimpleKey { id: 1 }, &10)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 3 }, &30)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 5 }, &50)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 7 }, &70)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_for_loop() {
        let mut map = SparseMap::<SimpleKey, i32, 16>::default();
        map.insert(SimpleKey { id: 3 }, 30);
        map.insert(SimpleKey { id: 1 }, 10);
        map.insert(SimpleKey { id: 5 }, 50);

        let mut count = 0;
        for (key, value) in map {
            match key {
                SimpleKey { id: 3 } => {
                    assert_eq!(key.id, 3);
                    assert_eq!(value, 30);
                }
                SimpleKey { id: 1 } => {
                    assert_eq!(key.id, 1);
                    assert_eq!(value, 10);
                }
                SimpleKey { id: 5 } => {
                    assert_eq!(key.id, 5);
                    assert_eq!(value, 50);
                }
                _ => unreachable!(),
            }
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn test_iter_mut_for_loop() {
        let mut map = SparseMap::<SimpleKey, u32, 1024>::default();
        map.insert(SimpleKey { id: 1 }, 10);
        map.insert(SimpleKey { id: 2 }, 20);
        map.insert(SimpleKey { id: 3 }, 30);

        for (_, value) in map.iter_mut() {
            *value += 1;
        }

        assert_eq!(map.get(SimpleKey { id: 1 }), Some(&11));
        assert_eq!(map.get(SimpleKey { id: 2 }), Some(&21));
        assert_eq!(map.get(SimpleKey { id: 3 }), Some(&31));
    }

    #[test]
    fn test_into_iter_mut_for_loop() {
        let mut map = SparseMap::<SimpleKey, u32, 1024>::default();
        map.insert(SimpleKey { id: 1 }, 10);
        map.insert(SimpleKey { id: 2 }, 20);
        map.insert(SimpleKey { id: 3 }, 30);

        for (_, value) in &mut map {
            *value += 1;
        }

        assert_eq!(map.get(SimpleKey { id: 1 }), Some(&11));
        assert_eq!(map.get(SimpleKey { id: 2 }), Some(&21));
        assert_eq!(map.get(SimpleKey { id: 3 }), Some(&31));
    }

    #[test]
    fn test_iter_mut() {
        let mut map = SparseMap::<SimpleKey, i32, 1024>::default();
        map.insert(SimpleKey { id: 1 }, 10);
        map.insert(SimpleKey { id: 3 }, 30);
        map.insert(SimpleKey { id: 5 }, 50);
        map.insert(SimpleKey { id: 7 }, 70);

        let mut iter = map.iter_mut();
        assert_eq!(iter.next(), Some((&SimpleKey { id: 1 }, &mut 10)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 3 }, &mut 30)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 5 }, &mut 50)));
        assert_eq!(iter.next(), Some((&SimpleKey { id: 7 }, &mut 70)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_get_existing_key() {
        let mut map = SparseMap::<SimpleKey, u32, 1024>::default();
        let key = SimpleKey { id: 42 };
        let value = 12345;
        map.insert(key, value);

        let result = map.get(key);
        assert_eq!(result, Some(&value));
    }

    #[test]
    fn test_get_nonexisting_key() {
        let mut map = SparseMap::<SimpleKey, u32, 1024>::default();
        let key = SimpleKey { id: 42 };
        let value = 12345;
        map.insert(key, value);

        let result = map.get(SimpleKey { id: 43 });
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_tombstone_key() {
        let mut map = SparseMap::<SimpleKey, u32, 1024>::default();
        let key = SimpleKey { id: 42 };
        let value = 12345;
        map.insert(key, value);

        let tombstone_key = SimpleKey::tombstone();

        let result = map.get(tombstone_key);
        assert_eq!(result, None);
    }
}
