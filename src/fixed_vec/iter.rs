use crate::FixedVec;
use std::alloc::{Layout, dealloc};
use std::mem::ManuallyDrop;
use std::ptr::{NonNull, drop_in_place, slice_from_raw_parts_mut};

impl<T> IntoIterator for FixedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = Self::IntoIter {
            ptr: self.ptr,
            len: self.len(),
            idx: 0,
            cap: self.capacity(),
        };

        let _ = ManuallyDrop::new(self);
        iter
    }
}

pub struct IntoIter<T> {
    ptr: NonNull<T>,
    len: usize,
    idx: usize,
    cap: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.len {
            return None;
        }

        unsafe {
            // SAFETY: we return if the index is out of bounds.
            let item_ptr = self.ptr.add(self.idx);
            self.idx += 1;
            Some(item_ptr.read())
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        // Drop any remaining initialized elements that haven't been yielded.
        if self.idx < self.len {
            let remaining = self.len - self.idx;
            unsafe {
                let start_ptr = self.ptr.as_ptr().add(self.idx);
                // SAFETY: elements in [idx, len) are initialized; we only drop them once here.
                let elems = slice_from_raw_parts_mut(start_ptr, remaining);
                drop_in_place(elems);
            }
        }

        // Deallocate the original allocation.
        let layout = Layout::array::<T>(self.cap).expect("Layout overflow");
        if layout.size() > 0 {
            unsafe {
                // SAFETY: we use the same layout as was used to allocate.
                dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}
