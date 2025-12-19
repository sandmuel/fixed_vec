mod fixed_vec;
pub use fixed_vec::{FixedVec, IntoIter};

#[cfg(test)]
mod tests {
    use crate::fixed_vec::FixedVec;

    #[test]
    fn single_thread() {
        let vec = FixedVec::<u64>::new(2);
        assert_eq!(Ok(()), vec.push(1));
        assert_eq!(Ok(()), vec.push(2));
        // No more space, the value should be returned.
        assert_eq!(Err(4), vec.push(4));
        // This should be in bounds.
        assert_eq!(Some(&2u64), vec.get(1));
        // This should be out of bounds.
        assert_eq!(None, vec.get(2));
        for item in vec.clone() {
            println!("{}", item);
        }
        for item in vec.into_iter().rev() {
            println!("{}", item);
        }
    }
}
