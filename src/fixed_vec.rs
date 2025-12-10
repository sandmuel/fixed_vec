use std::alloc::{Layout, alloc, handle_alloc_error, dealloc};
use std::hint::black_box;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// A thread safe [`Vec`]-like structure that never implicitly reallocates.
///
/// Because it uses atomics and does not reallocate, [`FixedVec::push`] does not require locks or a mutable reference to self.
pub struct FixedVec<T> {
    pointer: NonNull<T>,
    cap: usize,
    len: AtomicUsize,
}

// SAFETY: operations on the same value are atomic.
unsafe impl<T: Send> Send for FixedVec<T> {}
// SAFETY: addresses are all based on the atomic length and unmodified pointer. They cannot overlap.
unsafe impl<T: Sync> Sync for FixedVec<T> {}

impl<T> FixedVec<T> {
    pub fn new(mut capacity: usize) -> Self {
        let ptr;
        if capacity == 0 {
            ptr = NonNull::dangling();
        } else if size_of::<T>() == 0 {
            capacity = usize::MAX;
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
            pointer: ptr,
            cap: capacity,
            len: AtomicUsize::new(0),
        }
    }

    pub fn len(&self) -> usize {
        // Acquire to ensure writes up to this length have actually completed.
        self.len.load(Acquire)
    }

    pub fn acquire(&self) {
        // Acquire to ensure writes up to this length have actually completed.
        black_box(self.len.load(Acquire));
    }

    pub fn push(&self, value: T) -> Result<(), T> {
        let len = self.len.fetch_update(Release, Relaxed, |len| {
            if len < self.cap {
                return Some(len + 1);
            }
            None
        });

        match len {
            Ok(len) => {
                let ptr = unsafe { self.pointer.add(len) };
                unsafe { ptr.write(value) };
                Ok(())
            }
            Err(_) => Err(value),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            // SAFETY: index is within the length, so this is allocated and initialized memory.
            let ptr = unsafe { self.pointer.as_ptr().add(index) };
            // SAFETY: ptr was derived from a `NonNull`, so this shouldn't be null. It is aligned to `T`.
            return Some(unsafe { ptr.as_ref().expect("pointer should be non-null") });
        }
        None
    }
}

impl<T> Drop for FixedVec<T> {
    fn drop(&mut self) {
        unsafe {
            self.acquire();
            dealloc(self.pointer.as_ptr() as *mut u8, Layout::array::<T>(self.cap).unwrap());
        }
    }
}
