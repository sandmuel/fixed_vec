use crate::FixedVec;
use std::iter::FusedIterator;

impl<'a, T: Send + Sync> IntoIterator for &'a FixedVec<T> {
    type Item = &'a T;
    type IntoIter = IntoIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            vec: self,
            start: 0,
            end: self.len(),
        }
    }
}

pub struct IntoIter<'a, T> {
    vec: &'a FixedVec<T>,
    start: usize,
    end: usize,
}

impl<'a, T> Iterator for IntoIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }

        // SAFETY: we return if the index is out of bounds.
        let elem = unsafe { self.vec.get_unchecked(self.start) };
        self.start += 1;
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.end - self.start, Some(self.end - self.start))
    }

    // Manually implemented because we can do it faster since we know the length.
    fn count(self) -> usize {
        self.end - self.start
    }

    // Since we also implement DoubleEndedIterator, we use next_back for better
    // performance than the default implementation.
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<'_, T> {}

impl<T> FusedIterator for IntoIter<'_, T> {}

impl<T> DoubleEndedIterator for IntoIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end == self.start {
            return None;
        }

        // SAFETY: we return if the index is out of bounds.
        let elem = unsafe { self.vec.get_unchecked(self.end - 1) };
        self.end -= 1;
        Some(elem)
    }
}
