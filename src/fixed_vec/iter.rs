use crate::FixedVec;
use crate::fixed_vec::dealloc_vec;
use std::iter::FusedIterator;
use std::mem::ManuallyDrop;
use std::ptr::{NonNull, drop_in_place, slice_from_raw_parts_mut};

impl<T> IntoIterator for FixedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = Self::IntoIter {
            ptr: self.ptr,
            start: 0,
            end: self.len() - 1,
            cap: self.capacity(),
        };

        let _ = ManuallyDrop::new(self);
        iter
    }
}

pub struct IntoIter<T> {
    ptr: NonNull<T>,
    start: usize,
    end: usize,
    cap: usize,
}

// SAFETY: `T` is owned by `IntoIter` and provides no interior mutability of its
// own, so as long as `T` is Send, `IntoIter` is too.
unsafe impl<T: Send> Send for IntoIter<T> {}

// SAFETY: `IntoIter` has no public fields or methods which take `&self`.
unsafe impl<T> Sync for IntoIter<T> {}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        unsafe {
            // SAFETY: we return if the index is out of bounds.
            let item_ptr = self.ptr.add(self.start);
            self.start += 1;
            Some(item_ptr.read())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> FusedIterator for IntoIter<T> {}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end <= self.start {
            return None;
        }

        unsafe {
            // SAFETY: we return if the index is out of bounds.
            let item_ptr = self.ptr.add(self.end);
            self.end -= 1;
            Some(item_ptr.read())
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        struct DropGuard<'a, T>(&'a mut IntoIter<T>);

        impl<T> Drop for DropGuard<'_, T> {
            fn drop(&mut self) {
                dealloc_vec(self.0.ptr, self.0.cap);
            }
        }

        let _ = DropGuard(self);

        // Drop any remaining initialized elements that haven't been yielded.
        if self.start <= self.end {
            let remaining = self.end - self.start + 1;
            unsafe {
                let start_ptr = self.ptr.as_ptr().add(self.start);
                // SAFETY: elements in [idx, len) are initialized; we only drop them once here.
                let elems = slice_from_raw_parts_mut(start_ptr, remaining);
                drop_in_place(elems);
            }
        }

        // Deallocation occurs in DropGuard. This is called even if dropping
        // elements panics.
    }
}
