macro_rules! impl_from_variant {
    ($from:ty, $into:ty, $variant:ident $(,$($generic:ident),*)?) => {
        impl$(<$($generic)*>)? From<$from> for $into {
            fn from(value: $from) -> Self {
                <$into>::$variant(value)
            }
        }
    };
}

pub(crate) use impl_from_variant;
