use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::{NonNull, drop_in_place, slice_from_raw_parts_mut};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

mod iter;
mod iter_mut;
mod iter_ref;

pub use iter::IntoIter;

/// A thread safe [`Vec`]-like structure that never implicitly reallocates.
///
/// Because it uses atomics and does not reallocate, [`FixedVec::push`] does not
/// require locks or a mutable reference to self.
pub struct FixedVec<T> {
    ptr: NonNull<T>,
    next_idx: AtomicUsize,
    len: AtomicUsize,
    cap: usize,
}

// SAFETY: operations on the same value are atomic.
unsafe impl<T: Send> Send for FixedVec<T> {}
// SAFETY: addresses are all based on the atomic length and unmodified pointer.
// They cannot overlap.
unsafe impl<T: Sync> Sync for FixedVec<T> {}

impl<T> FixedVec<T> {
    pub fn new(capacity: usize) -> Self {
        let ptr;
        if capacity == 0 {
            ptr = NonNull::dangling();
        } else {
            let layout = Layout::array::<T>(capacity).expect("Layout overflow");

            // SAFETY: we check for a zero-sized type or capacity above.
            let raw_ptr = unsafe { alloc(layout) } as *mut T;

            if raw_ptr.is_null() {
                handle_alloc_error(layout);
            }

            // SAFETY: we check for a null pointer above.
            ptr = unsafe { NonNull::new_unchecked(raw_ptr) };
        }

        Self {
            ptr,
            next_idx: AtomicUsize::new(0),
            len: AtomicUsize::new(0),
            cap: capacity,
        }
    }

    pub fn realloc(&mut self) {
        let len = self.len();
        let new_vec = Self::new(len * 2);

        unsafe {
            new_vec.ptr.copy_from_nonoverlapping(self.ptr, len);
        }

        new_vec.next_idx.store(len, Relaxed);
        new_vec.len.store(len, Release);

        *self = new_vec;
    }

    #[inline]
    pub fn len(&self) -> usize {
        // Acquire to ensure writes up to this length have actually completed.
        self.len.load(Acquire)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    fn acquire(&self) {
        // Acquire to ensure writes up to this length have actually completed.
        self.len.load(Acquire);
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        // Using `Relaxed` since we don't care what goes on at previous indices when
        // pushing.
        let idx = self.next_idx.fetch_add(1, Relaxed);

        if idx < self.cap {
            unsafe {
                let ptr = self.ptr.add(idx);
                ptr.write(value);
            }
            loop {
                match self
                    .len
                    .compare_exchange_weak(idx, idx + 1, Release, Relaxed)
                {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
            Ok(())
        } else {
            Err(value)
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            // SAFETY: index is within the length, so this is allocated and initialized
            // memory. Allocations exceeding isize::MAX panic, so this can't overflow.
            let ptr = unsafe { self.ptr.add(index) };
            // SAFETY: ptr was derived from a `NonNull`, so this can't be null. It is
            // aligned to `T`.
            let elem = unsafe { ptr.as_ref() };
            return Some(elem);
        }
        None
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len() {
            // SAFETY: index is within the length, so this is allocated and initialized
            // memory.
            let mut ptr = unsafe { self.ptr.add(index) };
            // SAFETY: ptr was derived from a `NonNull`, so this can't be null, and, because
            // we use a mutable reference to self, it is exclusive. It is aligned to `T`.
            let elem = unsafe { ptr.as_mut() };
            return Some(elem);
        }
        None
    }
}

impl<T> Drop for FixedVec<T> {
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut FixedVec<T>);

        impl<T> Drop for DropGuard<'_, T> {
            fn drop(&mut self) {
                let layout = Layout::array::<T>(self.0.cap).unwrap();
                unsafe {
                    // Can't deallocate if it's zero-sized.
                    if layout.size() > 0 {
                        // SAFETY: the same layout was used to allocate.
                        dealloc(self.0.ptr.as_ptr() as *mut u8, layout);
                    }
                }
            }
        }

        let _ = DropGuard(self);

        // Drop elements.
        let elems = slice_from_raw_parts_mut(self.ptr.as_ptr(), self.len());
        unsafe {
            drop_in_place(elems);
        }

        // Deallocation occurs in DropGuard. This is called even if dropping
        // elements panics.
    }
}
