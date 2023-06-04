use std::env::VarError;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
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

macro_rules! loggers {
    ($(($name:literal, $macro:ident, $level:ident, $color:literal)),*) => {
        $(
            #[macro_export]
            macro_rules! $macro {
                ($msg:expr) => {{
                    use std::io::IsTerminal;

                    if $crate::util::log::log_level() <= $crate::util::log::LogLevel::$level {
                        let timestamp = chrono::offset::Utc::now();
                        match std::io::stdout().is_terminal() {
                            true => paris::log!(
                                "<{}> [<{}>{}</>] {}",
                                timestamp.format("%H:%M:%S"),
                                $color,
                                $name,
                                $msg
                            ),
                            false => paris::log!(
                                "<{}> [{}] {}",
                                timestamp.format("%Y-%m-%d %H:%M:%S"),
                                $name,
                                $msg
                            )
                        }
                    }
                }}
            }
        )*
    }
}

loggers!(
    ("DEBUG", debug, Debug, "cyan"),
    ("LOG", log, Info, "blue"),
    ("INFO", info, Info, "green"),
    ("WARN", warn, Warn, "yellow"),
    ("ERROR", error, Error, "red")
);
