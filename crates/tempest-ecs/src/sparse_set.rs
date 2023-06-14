use std::{
    alloc::{self, Layout},
    marker::PhantomData,
    mem::{needs_drop, size_of},
    ptr::{self, NonNull},
};

use super::sparse_index::SparseTableIndex;

pub struct SparseSet<T: SparseTableIndex, const PAGE_SIZE: usize> {
    packed: NonNull<T>,
    sparse: Vec<Box<[u32; PAGE_SIZE]>>,
    cap: usize,
    len: usize,
    marker: PhantomData<T>,
}

impl<T: SparseTableIndex, const PAGE_SIZE: usize> Default for SparseSet<T, PAGE_SIZE> {
    fn default() -> Self {
        Self {
            packed: NonNull::dangling(),
            sparse: Vec::default(),
            cap: 0,
            len: 0,
            marker: PhantomData,
        }
    }
}

impl<T: SparseTableIndex, const PAGE_SIZE: usize> Drop for SparseSet<T, PAGE_SIZE> {
    fn drop(&mut self) {
        if self.cap > 0 {
            // loop over packed storage and drop values
            if needs_drop::<T>() {
                for i in 0..self.len {
                    unsafe {
                        ptr::drop_in_place(self.packed.as_ptr().add(i));
                    }
                }
            }

            // release packed storage
            let packed_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.packed.as_ptr() as *mut u8, packed_layout);
            }

            self.sparse.clear();

            self.cap = 0;
            self.len = 0;
        }
    }
}

impl<T: SparseTableIndex, const PAGE_SIZE: usize> SparseSet<T, PAGE_SIZE> {
    pub fn insert(&mut self, value: T) {
        if self.len >= self.cap || value.index() as usize >= self.cap {
            self.grow_allocation(Some(value.index() as usize + 1));
        }

        let sparse_page_index = self.get_page(value);
        let sparse_page_offset = self.get_offset(value);

        let page = &mut self.sparse[sparse_page_index];
        let vacant = unsafe { *page.as_ptr().add(sparse_page_offset) == T::tombstone().index() };
        if vacant {
            page[sparse_page_offset] = self.len as u32;
            unsafe {
                ptr::write(self.packed.as_ptr().add(self.len), value);
            }
            self.len += 1;
        }
    }

    pub fn contains(&self, value: T) -> bool {
        let sparse_page_index = self.get_page(value);
        let sparse_page_offset = self.get_offset(value);
        if sparse_page_index < self.sparse.len() {
            unsafe {
                self.sparse[sparse_page_index]
                    .as_ptr()
                    .add(sparse_page_offset)
                    .as_ref()
                    .or(Some(&T::tombstone().index()))
                    .map(|v| !v.eq(&T::tombstone().index()))
                    .unwrap_or(false)
            }
        } else {
            false
        }
    }

    pub fn remove(&mut self, value: T) -> bool {
        let sparse_page_index = self.get_page(value);
        let sparse_page_offset = self.get_offset(value);

        if sparse_page_index < self.sparse.len() {
            let trampoline = self.sparse[sparse_page_index][sparse_page_offset];
            if trampoline != T::tombstone().index()
                && unsafe { *self.packed.as_ptr().add(trampoline as usize) }.eq(&value)
            {
                self.sparse[sparse_page_index][sparse_page_offset] = T::tombstone().index();

                unsafe {
                    let back = *self.packed.as_ptr().add(self.len - 1);

                    if needs_drop::<T>() {
                        ptr::drop_in_place(self.packed.as_ptr().add(self.len - 1));
                    }

                    *(self.packed.as_ptr().add(trampoline as usize)) = back;

                    self.len -= 1;

                    let (last_index, last_offset) = (self.get_page(back), self.get_offset(back));

                    if self.len != back.index() as usize {
                        self.sparse[last_index][last_offset] = trampoline;
                    }

                    true
                }
            } else {
                false
            }
        } else {
            false
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

        let new_layout = Layout::array::<T>(aligned_request).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            let old_ptr = self.packed.as_ptr() as *mut u8;

            unsafe {
                alloc::realloc(old_ptr, old_layout, aligned_request * size_of::<T>()) as *mut T
            }
        };

        self.packed = match NonNull::new(new_ptr as *mut T) {
            Some(p) => p,
            None => alloc::handle_alloc_error(new_layout),
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

        for _ in self.sparse.len()..requested_page_count {
            let page_ptr = Box::new([T::tombstone().index(); PAGE_SIZE]);
            self.sparse.push(page_ptr);
        }
    }

    fn get_page(&self, value: T) -> usize {
        value.index() as usize / PAGE_SIZE
    }

    fn get_offset(&self, value: T) -> usize {
        value.index() as usize % PAGE_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Index(u32);

    impl SparseTableIndex for Index {
        fn index(self) -> u32 {
            self.0
        }

        fn tombstone() -> Self {
            Index(u32::MAX)
        }

        fn from_raw(value: u32) -> Self {
            Index(value)
        }
    }

    #[test]
    fn test_insert_and_contains() {
        let mut set = SparseSet::<Index, 1024>::default();
        assert_eq!(set.contains(Index(0)), false);

        set.insert(Index(0));
        set.insert(Index(1));
        set.insert(Index(2));

        assert_eq!(set.contains(Index(0)), true);
        assert_eq!(set.contains(Index(1)), true);
        assert_eq!(set.contains(Index(2)), true);
        assert_eq!(set.contains(Index(3)), false);
    }

    #[test]
    fn test_remove() {
        let mut set = SparseSet::<Index, 1024>::default();
        set.insert(Index(0));
        set.insert(Index(1));

        assert_eq!(set.remove(Index(1)), true);
        assert_eq!(set.remove(Index(1)), false);
        assert_eq!(set.contains(Index(1)), false);
        assert_eq!(set.len(), 1);

        assert_eq!(set.remove(Index(0)), true);
        assert_eq!(set.remove(Index(0)), false);
        assert_eq!(set.contains(Index(0)), false);
        assert_eq!(set.len(), 0);
        assert_eq!(set.is_empty(), true);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut set = SparseSet::<Index, 1024>::default();
        assert_eq!(set.len(), 0);
        assert_eq!(set.is_empty(), true);

        set.insert(Index(0));
        assert_eq!(set.len(), 1);
        assert_eq!(set.is_empty(), false);

        set.remove(Index(0));
        assert_eq!(set.len(), 0);
        assert_eq!(set.is_empty(), true);
    }
}
