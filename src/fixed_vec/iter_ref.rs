use crate::{FixedVec, IntoIter};

impl<'a, T: Send + Sync> IntoIterator for &'a FixedVec<T> {
    type Item = &'a T;
    type IntoIter = IntoIter<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        unimplemented!()
    }
}
