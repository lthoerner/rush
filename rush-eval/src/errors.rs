use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("Command name could not be found as a builtin or an executable in PATH")]
    UnknownCommand(String),
}
