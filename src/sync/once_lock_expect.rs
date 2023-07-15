use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct OnceLockExpect<T>(OnceLock<T>);

impl<T> OnceLockExpect<T>
where
    T: Debug
{
    pub const fn new() -> OnceLockExpect<T> {
        OnceLockExpect(OnceLock::new())
    }

    pub fn set(once: &Self, value: T) {
        once.0
            .set(value)
            .expect("set should only be used initially")
    }

    pub fn is_initialized(once: &Self) -> bool {
        once.0.get().is_some()
    }
}

impl<T> Deref for OnceLockExpect<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get().expect("getting the value works after init")
    }
}
