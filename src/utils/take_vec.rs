pub trait TakeVec: Sized {
    fn take(&mut self) -> Self;
}

impl<T> TakeVec for Vec<T>
where
    T: Copy,
{
    fn take(&mut self) -> Self {
        // TODO might be able to optimize this with unsafe functions?
        let mut result = Vec::with_capacity(self.len());
        for elem in self.iter().rev() {
            result.push(*elem);
        }

        self.clear();

        result
    }
}
