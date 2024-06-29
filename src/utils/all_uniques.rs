use std::collections::HashSet;
use std::hash::Hash;

pub trait AllUnique {
    fn all_unique(self) -> bool;
}

impl<I> AllUnique for I
where
    I: Iterator<Item: Eq + Hash>,
{
    fn all_unique(mut self) -> bool {
        let mut set_of_uniques = HashSet::new();
        self.all(|elem| set_of_uniques.insert(elem))
    }
}
