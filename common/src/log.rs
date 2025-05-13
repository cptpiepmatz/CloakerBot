#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3
}

impl Default for LogLevel {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return LogLevel::Debug;

        #[cfg(not(debug_assertions))]
        return LogLevel::Info;
    }
}

pub fn log_level() -> LogLevel {
    match std::env::var("LOG_LEVEL") {
        Ok(level) => match level.to_lowercase().as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::default()
        },
        _ => LogLevel::default()
    }
}
