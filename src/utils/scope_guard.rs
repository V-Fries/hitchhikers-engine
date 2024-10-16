use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

#[macro_export]
macro_rules! defer {
    ($($t:tt)*) => {
        let _score_guard = $crate::utils::ScopeGuard::new((), |()| { $($t)* });
    };
}

#[allow(dead_code)]
pub trait Defer: Sized {
    fn defer<F>(self, callback: F) -> ScopeGuard<Self, F>
    where
        F: FnOnce(Self),
    {
        ScopeGuard::new(self, callback)
    }
}

impl<T> Defer for T {}

pub struct ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    content: ManuallyDrop<T>,
    callback: ManuallyDrop<F>,
}

impl<T, F> ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    #[allow(dead_code)]
    pub fn new(content: T, callback: F) -> Self {
        ScopeGuard {
            content: ManuallyDrop::new(content),
            callback: ManuallyDrop::new(callback),
        }
    }

    #[allow(dead_code)]
    pub fn into_inner(mut scope_guard: Self) -> T {
        unsafe {
            ManuallyDrop::drop(&mut scope_guard.callback);
            let content = ManuallyDrop::take(&mut scope_guard.content);
            let _ = ManuallyDrop::new(scope_guard);
            content
        }
    }
}

impl<T, F> Deref for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    type Target = T;

    fn deref(&self) -> &T {
        &self.content
    }
}

impl<T, F> DerefMut for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    fn deref_mut(&mut self) -> &mut T {
        &mut self.content
    }
}

impl<T, F> Drop for ScopeGuard<T, F>
where
    F: FnOnce(T),
{
    fn drop(&mut self) {
        unsafe {
            let callback = ManuallyDrop::take(&mut self.callback);
            let content = ManuallyDrop::take(&mut self.content);
            callback(content);
        }
    }
}
