#[macro_export]
macro_rules! impl_from_variant {
    ($from:ty, $into:ty, $variant:ident) => {
        impl From<$from> for $into {
            fn from(value: $from) -> Self {
                <$into>::$variant(value)
            }
        }
    };
}
