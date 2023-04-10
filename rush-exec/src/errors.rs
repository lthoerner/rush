use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuiltinError {
    #[error("Wrong number of arguments")]
    InvalidArgumentCount,
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Invalid value for argument")]
    InvalidValue,
    // $ This is way too general
    #[error("Runtime error")]
    FailedToRun,
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Failed to execute external command")]
    FailedToExecute(isize),
}
