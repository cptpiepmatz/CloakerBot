#[macro_export]
macro_rules! lazy_env {
    ($env:literal) => {
        ::std::sync::LazyLock::new(|| {
            use crate::diagnostics::panic::ExpectReport;
            ::std::env::var($env).expect_report(concat!("could not read env variable ", $env))
        })
    };
}
