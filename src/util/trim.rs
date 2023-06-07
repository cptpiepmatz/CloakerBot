#[macro_export]
macro_rules! trim {
    ($in:expr) => {
        str_replace!(str_replace!(str_replace!($in, " ", ""), "\r", ""), "\n", "")
    };
}
