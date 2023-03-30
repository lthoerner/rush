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
    #[error("Failed to flush stdout")]
    FailedToFlushStdout,
    #[error("Failed to read from stdin")]
    FailedToReadStdin,
    #[error("Unknown error")]
    Uncategorized,
}
