macro_rules! trim {
    ($in:expr) => {
        konst::string::trim($in)
    };
}

pub(crate) use trim;
