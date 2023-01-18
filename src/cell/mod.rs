use std::cell::UnsafeCell;
use std::ops::Deref;

pub struct InitCell<T> {
    value: UnsafeCell<Option<T>>
}

impl<T> InitCell<T> {
    pub const fn new() -> InitCell<T> {
        InitCell {
            value: UnsafeCell::new(None)
        }
    }

    pub fn init(cell: &InitCell<T>, value: T) {
        let inner = unsafe { &mut *cell.value.get() };
        match inner {
            Some(_v) => panic!("already initialized"),
            None => inner.insert(value)
        };
    }
}

impl<T> Deref for InitCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner = unsafe { &mut *self.value.get() };
        match inner {
            Some(v) => v,
            None => panic!("not initialized")
        }
    }
}

unsafe impl<T> Sync for InitCell<T> where T: Sync + Send {}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use crate::cell::InitCell;

    #[test]
    #[should_panic]
    fn panic_if_not_init() {
        let cell: InitCell<()> = InitCell::new();
        let _ = cell.deref();
    }

    #[test]
    #[should_panic]
    fn panic_if_double_assignment() {
        let cell: InitCell<()> = InitCell::new();
        InitCell::init(&cell, ());
        InitCell::init(&cell, ());
    }

    #[test]
    fn deref_value() {
        let cell: InitCell<()> = InitCell::new();
        InitCell::init(&cell, ());
        let _: &() = cell.deref();
    }

    #[test]
    fn values_identical() {
        let cell: InitCell<char> = InitCell::new();
        InitCell::init(&cell, 'x');
        let left = cell.deref();
        let right = cell.deref();
        assert_eq!(left, right);
    }
}
