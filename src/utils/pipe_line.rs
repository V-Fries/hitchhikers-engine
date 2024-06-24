pub trait PipeLine: Sized {
    fn pipe<T, F>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
    {
        f(self)
    }
}

impl<T> PipeLine for T {}
