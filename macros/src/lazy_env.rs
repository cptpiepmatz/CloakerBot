#[macro_export]
#[allow(clippy::crate_in_macro_def, reason = "diagnostics belong to the calling crate")]
macro_rules! lazy_env {
    ($env:literal) => {
        ::std::sync::LazyLock::new(|| {
            use crate::diagnostics::panic::ExpectReport;
            ::std::env::var($env).expect_report(concat!("could not read env variable ", $env))
        })
    };
}
