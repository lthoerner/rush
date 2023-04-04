#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Failed to get one or more external environment variables")]
    MissingExternalEnvironmentVariables,
    #[error("Failed to get one or more internal environment variables")]
    MissingInternalEnvironmentVariables,
    #[error("Failed to update one or more shell environment variables")]
    FailedToUpdateEnvironmentVariables,
    #[error("Failed to convert PathBuf to String")]
    FailedToConvertPathBufToString,
    #[error("Failed to canonicalize path")]
    FailedToCanonicalizePath,
    #[error("Failed to flush stdout")]
    FailedToFlushStdout,
    #[error("Failed to read from stdin")]
    FailedToReadStdin,
    #[error("Directory does not exist")]
    UnknownDirectory,
    #[error("Unknown error")]
    Uncategorized,
}

#[derive(Error, Debug)]
pub enum InternalCommandError {
    #[error("Wrong number of arguments")]
    InvalidArgumentCount,
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Invalid value for argument")]
    InvalidValue,
    // * This might be too general, might be better to do error variants like "FailedToOpenFile" etc
    #[error("Runtime error")]
    FailedToRun,
}

#[derive(Error, Debug)]
pub enum ExternalCommandError {
    #[error("Failed to execute external command")]
    FailedToExecute(isize),
    #[error("Failed to read from stdout")]
    FailedToReadStdout,
    #[error("Failed to read from stderr")]
    FailedToReadStderr,
}
