use std::collections::HashSet;
use std::hash::Hash;

pub trait GetAllUniques<T> {
    fn get_all_uniques<R>(self) -> R
        where
            R: FromIterator<T>;
}

impl<I, T> GetAllUniques<T> for I
    where
        I: Iterator<Item=T>,
        T: Eq + Hash {
    fn get_all_uniques<R>(self) -> R
        where
            R: FromIterator<T> {
        self.collect::<HashSet<T>>().into_iter().collect()
    }
}
