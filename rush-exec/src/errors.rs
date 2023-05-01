use std::path::PathBuf;
use std::{fmt, io};

use thiserror::Error;

/// This is a wrapper for io::Error to add more context than the default Display.
/// It should not be used directly. Use an internal error instead.
#[derive(Error, Debug)]
pub struct IoError {
    #[from]
    source: io::Error,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)?;
        if let Some(e) = self.source.get_ref() {
            if let Some(e) = e.source() {
                write!(f, " because: {}", e)?;
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum BuiltinError {
    #[error("Wrong number of arguments: {0}")]
    InvalidArgumentCount(usize),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid value for argument: {0}")]
    InvalidValue(String),
    // $ This is way too general
    #[error("Runtime error")]
    FailedToRun,
    #[error("Unable to read Path: {0}")]
    FailedReadingPath(PathBuf),
    #[error("Unable to read file type from path: {0}")]
    FailedReadingFileType(PathBuf),
    #[error("Unable to read file name from path: {0}")]
    FailedReadingFileName(PathBuf),
    #[error("Unable to read dir: {0}")]
    FailedReadingDir(PathBuf),
    #[error("{0}")]
    IoError(#[from] IoError),
}

impl From<io::Error> for BuiltinError {
    fn from(source: io::Error) -> Self {
        Self::IoError(source.into())
    }
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Path no longer exists: {0}")]
    PathNoLongerExists(PathBuf),
    #[error("Executable failed with exit code: {0}")]
    FailedToExecute(isize),
}
