#[macro_export]
macro_rules! map {
    ($($key:tt$(: $value:expr)?),* $(,)?) => {
        ::std::collections::HashMap::from_iter([
            $($crate::map!(@entry: $key $(: $value)?)),*
        ])
    };

    (@entry: $key:ident) => {(::std::stringify!($key), $key)};
    (@entry: $key:ident: $value:expr) => {(::std::stringify!($key), $value)};
    (@entry: $key:literal: $value:expr) => {($key, $value)};
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    fn expected<K: Eq + std::hash::Hash, V>(pairs: Vec<(K, V)>) -> HashMap<K, V> {
        HashMap::from_iter(pairs)
    }

    #[test]
    fn test_ident_key_same_value() {
        let foo = 123;
        let bar = 456;

        let result = map! {foo, bar};
        let expected = expected(vec![("foo", 123), ("bar", 456)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_ident_key_custom_value() {
        let result = map! {
            foo: 1,
            bar: 2
        };
        let expected = expected(vec![("foo", 1), ("bar", 2)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_literal_key() {
        let result = map! {
            "a": 10,
            "b": 20
        };
        let expected = expected(vec![("a", 10), ("b", 20)]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_keys() {
        let x = 7;
        let result = map! {
            x,
            y: 9,
            "z": 11
        };
        let expected = expected(vec![("x", 7), ("y", 9), ("z", 11)]);

        assert_eq!(result, expected);
    }
}
