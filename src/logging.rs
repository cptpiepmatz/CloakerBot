use std::io::IsTerminal;

use thiserror::Error;
use tracing_subscriber::EnvFilter;

#[cfg(debug_assertions)]
const DEFAULT_FILTER: &'static str = "warn,CloakerBot=debug";
#[cfg(not(debug_assertions))]
const DEFAULT_FILTER: &'static str = "warn,CloakerBot=info";

const _: () = assert!(konst::string::eq_str(EnvFilter::DEFAULT_ENV, "RUST_LOG"));

#[derive(Debug, Error)]
pub enum InitLoggingError {
    #[error("could not read {name}")]
    Env {
        name: &'static str,
        #[source]
        source: std::env::VarError
    },
    #[error("invalid tracing filter")]
    Filter(#[from] tracing_subscriber::filter::ParseError),
    #[error("LOG_FORMAT must be text or json, got {0:?}")]
    Format(String),
    #[error("could not initialize tracing")]
    Subscriber(#[source] Box<dyn std::error::Error + Send + Sync>)
}

pub fn init() -> Result<(), InitLoggingError> {
    let filter = match std::env::var("RUST_LOG") {
        Ok(value) => EnvFilter::try_new(value)?,
        Err(std::env::VarError::NotPresent) => EnvFilter::try_new(DEFAULT_FILTER)?,
        Err(source) => return Err(InitLoggingError::Env { name: "RUST_LOG", source })
    };
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::rfc_3339())
        .with_writer(std::io::stderr);
    match std::env::var("LOG_FORMAT").as_deref() {
        Ok("json") => subscriber.json().try_init(),
        Ok("text") | Err(std::env::VarError::NotPresent) => {
            subscriber.with_ansi(std::io::stderr().is_terminal()).try_init()
        }
        Ok(value) => return Err(InitLoggingError::Format(value.to_owned())),
        Err(source) => {
            return Err(InitLoggingError::Env { name: "LOG_FORMAT", source: source.clone() });
        }
    }
    .map_err(InitLoggingError::Subscriber)?;
    Ok(())
}
