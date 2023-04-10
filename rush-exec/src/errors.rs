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
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Executable failed with exit code: {0}")]
    FailedToExecute(isize),
}
