use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShellError {
    #[error("Failed to get one or more external environment variables")]
    MissingExternalEnvironmentVariables,
    #[error("Failed to get one or more internal environment variables")]
    MissingInternalEnvironmentVariables,
    #[error("Failed to update one or more shell environment variables")]
    FailedToUpdateEnvironmentVariables,
    #[error("Previous directory does not exist")]
    NoPreviousDirectory,
    #[error("Next directory does not exist")]
    NoNextDirectory,
    #[error("Failed to open configuration file")]
    FailedToOpenConfigFile,
    #[error("Failed to read configuration file")]
    FailedToReadConfigFile,
    #[error("Unknown error")]
    Uncategorized,
}

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Failed to convert PathBuf to String")]
    FailedToConvertPathBufToString,
    #[error("Failed to canonicalize directory path")]
    FailedToCanonicalize,
    #[error("Failed to access directory path")]
    FailedToAccess,
    #[error("Directory does not exist")]
    UnknownDirectory,
}
