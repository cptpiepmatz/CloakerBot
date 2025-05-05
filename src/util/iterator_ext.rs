pub trait IteratorExt: Iterator {
    /// Finds an element in the iterator that satisfies a predicate and returns the last element.
    ///
    /// This combines the behavior of [Iterator::find] and [Iterator::last].
    /// Using both of these methods separately can cause issues:
    /// after using `find`, the iterator will be empty, and using `last` afterward will yield
    /// nothing, and vice versa.
    ///
    /// The method has four possible outcomes:
    /// - `(None, None)`: The iterator was empty, no element was found, and there is no last element.
    /// - `(Some, Some)`: An element was found, but it wasn't the last one.
    /// - `(Some, None)`: The element was found, and it was the last one.
    /// - `(None, Some)`: No element was found, but the iterator wasn't empty.
    fn find_and_last<P>(&mut self, predicate: P) -> (Option<Self::Item>, Option<Self::Item>)
    where
        P: FnMut(&Self::Item) -> bool;
}

impl<I> IteratorExt for I
where
    I: Iterator,
{
    fn find_and_last<P>(&mut self, mut predicate: P) -> (Option<Self::Item>, Option<Self::Item>)
    where
        P: FnMut(&Self::Item) -> bool,
    {
        let mut found = None;
        let mut last = None;

        while let Some(next) = self.next() {
            match predicate(&next) {
                false => last = Some(next),
                true => {
                    found = Some(next);
                    last = None;
                }
            }
        }

        (found, last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_and_last_found_not_last() {
        let vec = vec![1, 2, 3, 4];
        let result = vec.into_iter().find_and_last(|&x| x == 2);
        assert_eq!(result, (Some(2), Some(4)));
    }

    #[test]
    fn test_find_and_last_found_is_last() {
        let vec = vec![1, 2, 3, 4];
        let result = vec.into_iter().find_and_last(|&x| x == 4);
        assert_eq!(result, (Some(4), None));
    }

    #[test]
    fn test_find_and_last_not_found() {
        let vec = vec![1, 2, 3, 4];
        let result = vec.into_iter().find_and_last(|&x| x == 5);
        assert_eq!(result, (None, Some(4)));
    }

    #[test]
    fn test_find_and_last_empty() {
        let vec: Vec<i32> = Vec::new();
        let result = vec.into_iter().find_and_last(|&x| x == 2);
        assert_eq!(result, (None, None));
    }
}

