use crate::FixedVec;
use std::ptr::NonNull;

impl<T: Send + Sync> IntoIterator for FixedVec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            ptr: self.ptr,
            len: self.len(),
            idx: 0,
        }
    }
}

pub struct IntoIter<T: Send + Sync> {
    ptr: NonNull<T>,
    len: usize,
    idx: usize,
}

impl<T: Send + Sync> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let ptr = unsafe { self.ptr.add(self.idx) };
        }
        None
    }
}
