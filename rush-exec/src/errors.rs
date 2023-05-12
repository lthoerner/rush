use std::path::PathBuf;

use thiserror::Error;

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
}
