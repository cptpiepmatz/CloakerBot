#[macro_export]
macro_rules! trim {
    ($in:expr) => {
        ::konst::string::trim($in)
    };
}
