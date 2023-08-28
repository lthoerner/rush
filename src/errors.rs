use std::path::PathBuf;

use thiserror::Error;

use crate::state::environment::EnvVariable;

#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("Command name could not be found as a builtin or an executable in PATH")]
    UnknownCommand(String),
    #[error("Command does not have the executable permissions set. Current permissions are: {0}")]
    CommandNotExecutable(u32),
    #[error("Failed to read metadata for executable: {0}")]
    FailedToReadExecutableMetadata(String),
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
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Path no longer exists: {0}")]
    PathNoLongerExists(PathBuf),
    #[error("Executable failed with exit code: {0}")]
    FailedToExecute(isize),
}

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Failed to get external environment variable: {0}")]
    MissingExternalEnvironmentVariable(EnvVariable),
    #[error("Failed to get internal environment variable: {0}")]
    MissingInternalEnvironmentVariable(EnvVariable),
    #[error("Failed to update shell environment variable: {0}")]
    FailedToUpdateEnvironmentVariable(EnvVariable),
    #[error("Previous directory does not exist")]
    NoPreviousDirectory,
    #[error("Next directory does not exist")]
    NoNextDirectory,
    #[error("Failed to open configuration file: {0}")]
    FailedToOpenConfigFile(PathBuf),
    #[error("Failed to read configuration file: {0}")]
    FailedToReadConfigFile(PathBuf),
    #[error("Unknown error")]
    Uncategorized,
}

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Failed to convert PathBuf to String: {0}")]
    FailedToConvertPathBufToString(PathBuf),
    #[error("Failed to canonicalize directory path: {0}")]
    FailedToCanonicalize(PathBuf),
    #[error("Failed to access directory path: {0}")]
    FailedToAccess(PathBuf),
    #[error("Directory does not exist: {0}")]
    UnknownDirectory(PathBuf),
}
