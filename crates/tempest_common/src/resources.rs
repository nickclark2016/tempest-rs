//! Handles used for reference and ownership of application resources.

use std::{fmt::Debug, hash::Hash, marker::PhantomData, ops::Deref, sync::Arc};

/// Resource handle that does not own the underlying resource.  Resource handle
/// does not directly contain the underlying data, but represents a shared
/// ownership model.
pub struct WeakResourceHandle<T> {
    /// Identifier of the resource handle
    pub idx: usize,
    _type: PhantomData<T>,
}

impl<T> WeakResourceHandle<T> {
    /// Constructs a new weak resource handle with the provided identifier.
    pub const fn new(idx: usize) -> Self {
        Self {
            idx,
            _type: PhantomData,
        }
    }
}

impl<T> Debug for WeakResourceHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakResourceHandle")
            .field("idx", &self.idx)
            .finish()
    }
}

impl<T> Copy for WeakResourceHandle<T> {}

impl<T> Clone for WeakResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            _type: PhantomData,
        }
    }
}

impl<T> PartialEq for WeakResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx
    }
}

impl<T> Eq for WeakResourceHandle<T> {}

impl<T> PartialOrd for WeakResourceHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}

impl<T> Ord for WeakResourceHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

impl<T> Hash for WeakResourceHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}

/// Function to be invoked on resource usage drop to zero
type ResourceDropFn<T> = dyn Fn(WeakResourceHandle<T>) + Send + Sync;

/// Resource handle that does owns the underlying resource.  Resource handle
/// does not directly contain the underlying data, but represents an owner in a
/// shared ownership model.
pub struct ResourceHandle<T> {
    ref_count: Arc<ResourceDropFn<T>>,
    internal: WeakResourceHandle<T>,
    _type: PhantomData<T>,
}

impl<T> Drop for ResourceHandle<T> {
    fn drop(&mut self) {
        if Arc::strong_count(&self.ref_count) == 1 {
            (self.ref_count)(self.internal);
        }
    }
}

impl<T> Debug for ResourceHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("ref_count", &Arc::strong_count(&self.ref_count))
            .field("idx", &self.internal.idx)
            .finish()
    }
}

impl<T> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            ref_count: self.ref_count.clone(),
            internal: self.internal,
            _type: PhantomData,
        }
    }
}

impl<T> PartialEq for ResourceHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.internal.idx == other.internal.idx
    }
}

impl<T> Eq for ResourceHandle<T> {}

impl<T> PartialOrd for ResourceHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.internal.idx.partial_cmp(&other.internal.idx)
    }
}

impl<T> Ord for ResourceHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.internal.idx.cmp(&other.internal.idx)
    }
}

impl<T> Hash for ResourceHandle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.internal.idx.hash(state);
    }
}

impl<T> ResourceHandle<T> {
    /// Constructs a new resource handle given an identifier and a function to execute on drop.
    pub fn new(
        destructor: impl Fn(WeakResourceHandle<T>) + Send + Sync + 'static,
        idx: usize,
    ) -> Self {
        Self {
            ref_count: Arc::new(destructor),
            internal: WeakResourceHandle::new(idx),
            _type: PhantomData,
        }
    }

    /// Gets the internal weak resource handle owned by this handle
    pub fn get_internal(&self) -> WeakResourceHandle<T> {
        self.internal
    }
}

impl<T> Deref for ResourceHandle<T> {
    type Target = WeakResourceHandle<T>;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}
