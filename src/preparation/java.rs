use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::Error as IoError;
use std::path::PathBuf;

use java_locator::errors::JavaLocatorError;
use java_locator::locate_java_home;

#[derive(Debug)]
pub enum FindJavaError {
    JavaHome(JavaLocatorError),
    Io(IoError),
    Missing(FileMissingError)
}

impl Display for FindJavaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FindJavaError::JavaHome(err) => Display::fmt(err, f),
            FindJavaError::Io(err) => Display::fmt(err, f),
            FindJavaError::Missing(err) => Display::fmt(err, f)
        }
    }
}

impl Error for FindJavaError {}

impl From<JavaLocatorError> for FindJavaError {
    fn from(value: JavaLocatorError) -> Self {
        FindJavaError::JavaHome(value)
    }
}

#[derive(Debug)]
pub struct FileMissingError {
    expected: PathBuf
}

impl Display for FileMissingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected file at path {}",
            self.expected.to_string_lossy()
        )
    }
}

impl Error for FileMissingError {}

pub fn find_java_executable() -> Result<PathBuf, FindJavaError> {
    let java_home = locate_java_home()?;
    let mut java_exec = PathBuf::from(java_home);
    java_exec.push("bin");
    java_exec.push("java");
    #[cfg(windows)]
    java_exec.set_extension("exe");
    match java_exec.try_exists() {
        Ok(true) => Ok(java_exec),
        Ok(false) => Err(FindJavaError::Missing(FileMissingError {
            expected: java_exec
        })),
        Err(err) => Err(FindJavaError::Io(err))
    }
}
