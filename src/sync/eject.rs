use parking_lot::Mutex;

/// A type that holds a value and allows it to be ejected once using a shared
/// reference.
///
/// `Eject` is designed to hold a value that can be extracted exactly once.
/// By using a shared reference to the `Eject`, the value can be ejected and
/// returned, leaving it in a "taken" state.
/// This is ideal for managing global state where only one component is allowed
/// to extract the value, while other parts of the code can continue to access
/// the state without issues.
#[derive(Debug)]
pub struct Eject<T>(Mutex<Option<T>>);

impl<T> Eject<T> {
    /// Creates a new `Eject` with the specified value.
    pub fn new(inner: T) -> Self {
        Self(Mutex::new(Some(inner)))
    }

    /// Extracts the inner value once.
    ///
    /// Returning `Some(T)` on the first call, and `None` on subsequent calls
    /// after the value is ejected.
    #[must_use]
    pub fn eject(&self) -> Option<T> {
        self.0.lock().take()
    }
}
