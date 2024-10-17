pub trait PipeLine: Sized {
    fn pipe<T, F>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
    {
        f(self)
    }

    #[allow(dead_code)]
    unsafe fn pipe_unsafe<T>(self, f: unsafe fn(Self) -> T) -> T {
        f(self)
    }
}

impl<T> PipeLine for T {}
