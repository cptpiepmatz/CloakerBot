use std::fmt::Debug;
use std::ops::Deref;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct OnceLockExpect<T>(OnceLock<T>);

impl<T> OnceLockExpect<T> {
    pub const fn new() -> OnceLockExpect<T> {
        OnceLockExpect(OnceLock::new())
    }

    pub fn set(once: &Self, value: T) {
        if let Err(_) = once.0.set(value) {
            panic!("OnceLockExpect::set called while already set");
        }
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
