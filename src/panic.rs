use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use std::panic::Location;

use miette::{Diagnostic, Report};

#[derive(Debug, Diagnostic)]
#[diagnostic(forward(report), help("panic originated at {location}"))]
struct PanicDiagnostic {
    thread: String,
    location: &'static Location<'static>,
    report: Report
}

impl Display for PanicDiagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "thread '{}' panicked", self.thread)
    }
}

impl Error for PanicDiagnostic {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(AsRef::<dyn Error>::as_ref(&self.report))
    }
}

pub fn init() {
    miette::set_panic_hook();
    let fallback = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        match info.payload().downcast_ref::<Report>() {
            Some(report) => writeln!(std::io::stderr(), "{report:?}").unwrap_or_else(|_| ()),
            None => fallback(info)
        }
    }));
}

#[track_caller]
#[cold]
pub fn panic_report(report: Report) -> ! {
    let thread = std::thread::current().name().unwrap_or("unnamed").to_owned();
    let report = Report::new(PanicDiagnostic { thread, location: Location::caller(), report });
    std::panic::panic_any(report)
}

/// Extracts a value or panics with a diagnostic report at the caller's
/// location.
pub trait ExpectReport<T> {
    #[track_caller]
    fn expect_report(self, message: impl Into<String>) -> T;
}

impl<T, E> ExpectReport<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static
{
    #[track_caller]
    fn expect_report(self, message: impl Into<String>) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic_report(Report::from_err(error).wrap_err(message.into()))
        }
    }
}

impl<T> ExpectReport<T> for Option<T> {
    #[track_caller]
    fn expect_report(self, message: impl Into<String>) -> T {
        match self {
            Some(value) => value,
            None => panic_report(miette::miette!("{}", message.into()))
        }
    }
}
