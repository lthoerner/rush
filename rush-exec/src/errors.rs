use std::io::ErrorKind;
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
    #[error("Could not find file.")]
    FileNotFound,
    #[error("Insufficient permissions to read file.")]
    NoReadPermissions,
    /// This variant is a fallthrough, and you should generally prefer a more specific/human-readable error
    #[error("{0}")]
    OtherIoError(#[from] IoError),
}

impl BuiltinError {
    pub fn read_file(source: io::Error) -> Self {
        match source.kind() {
            // unstable: ErrorKind::IsADirectory => Self::OtherIoError(source.into()),
            ErrorKind::NotFound => Self::FileNotFound,
            ErrorKind::PermissionDenied => Self::NoReadPermissions,
            _ => Self::OtherIoError(source.into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Path no longer exists: {0}")]
    PathNoLongerExists(PathBuf),
    #[error("Executable failed with exit code: {0}")]
    FailedToExecute(isize),
    #[error("Failed to parse executable stdout: {0}")]
    FailedToParseStdout(String),
    #[error("Failed to parse executable stderr: {0}")]
    FailedToParseStderr(String),
    /// This variant is a fallthrough, and you should generally prefer a more specific/human-readable error
    #[error("{0}")]
    OtherIoError(#[from] IoError),
}

impl ExecutableError {
    pub fn unexpected(source: io::Error) -> Self {
        Self::OtherIoError(source.into())
    }
}
