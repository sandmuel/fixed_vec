mod fixed_vec;
pub use fixed_vec::FixedVec;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::fixed_vec::FixedVec;
    use super::*;

    #[test]
    fn it_works() {
        let vec = FixedVec::<u64>::new(3);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(4);
    }
}
