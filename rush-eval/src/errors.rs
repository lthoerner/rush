use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("Command name could not be found as a builtin or an executable in PATH")]
    UnknownCommand(String),
    #[error("Command does not have the executable permissions set. Current permissions are: {0}")]
    CommandNotExecutable(u32),
    #[error("Failed to read metadata for executable: {0}")]
    FailedToReadExecutableMetadata(String),
}
