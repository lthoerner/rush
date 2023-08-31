use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::state::EnvVariable;

/// `Result` alias which automatically uses `RushError` as the error type.
pub type Result<T> = std::result::Result<T, RushError>;
pub trait Handle<T> {
    /// Replaces any error kind with a new one, without overriding the default error message.
    /// Useful in situations where additional context provides no additional clarity.
    fn replace_err(self, new_error: RushError) -> Result<T>;
    /// Replaces any error kind with a new one, overriding the default error message with the
    /// provided one. Useful in situations where additional context can provide additional clarity.
    fn replace_err_with_msg(self, new_error: RushError, context: &str) -> Result<T>;
}

impl<T, E> Handle<T> for std::result::Result<T, E> {
    fn replace_err(mut self, new_error: RushError) -> Result<T> {
        self.map_err(|_| new_error)
    }

    fn replace_err_with_msg(mut self, new_error: RushError, context: &str) -> Result<T> {
        self.map_err(|_| new_error.set_context(context))
    }
}

impl<T> Handle<T> for std::option::Option<T> {
    fn replace_err_with_msg(mut self, new_error: RushError, context: &str) -> Result<T> {
        self.ok_or(new_error.set_context(context))
    }

    fn replace_err(mut self, new_error: RushError) -> Result<T> {
        self.ok_or(new_error)
    }
}

/// Error type for Rush.
/// Contains an error kind and optionally a custom message,
/// which is used to override the default error message.
/// All error kinds have a default error message and an extended description of the error,
/// including a detailed explanation of what the error kind represents and potential causes.
pub struct RushError {
    kind: ErrorKind,
    custom_message: Option<String>,
}

impl Display for RushError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // If the error has a custom message, use it instead of the default error message.
        write!(
            f,
            "{}",
            self.custom_message.unwrap_or(self.kind.to_string())
        )
    }
}

impl RushError {
    /// Creates a `RushError` with no custom message.
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            custom_message: None,
        }
    }

    /// Takes a `RushError` and gives it a custom message.
    pub fn set_context(mut self, context: &str) -> Self {
        self.custom_message = Some(context.to_owned());
        self
    }
}

/// Enum representing every type of error which can occur in Rush.
/// Downstream error variants will typically include data providing basic information
/// about how the error occurred, such as the name of a command which was not found.
pub enum ErrorKind {
    Dispatch(DispatchError),
    Builtin(BuiltinError),
    Executable(ExecutableError),
    State(StateError),
    Path(PathError),
}

/// Error type for errors which occur during command dispatch.
pub enum DispatchError {
    UnknownCommand(String),
    CommandNotExecutable(u32),
    UnreadableExecutableMetadata(PathBuf),
}

/// Error type for errors which occur during execution of builtins.
pub enum BuiltinError {
    WrongArgCount(usize, usize),
    InvalidArg(String),
    InvalidValue(String),
    UnreadableFileType(PathBuf),
    UnreadableFileName(PathBuf),
    UnreadableDirectory(PathBuf),
    // TODO: Break this into multiple error types
    FailedToRun,
}

/// Error type for errors which occur during execution of executable files.
pub enum ExecutableError {
    PathNoLongerExists(PathBuf),
    FailedToExecute(isize),
    CouldNotWait,
}

/// Error type for errors which occur during state operations.
pub enum StateError {
    MissingEnv(EnvVariable),
    CouldNotUpdateEnv(EnvVariable),
    NoPreviousDirectory,
    NoNextDirectory,
    UnopenableConfig(PathBuf),
    UnreadableConfig(PathBuf),
    UnsupportedTerminal,
}

/// Error type for errors which occur during path operations.
pub enum PathError {
    FailedToConvertStringToPath(String),
    FailedToConvertPathToString(PathBuf),
    CouldNotCanonicalize(PathBuf),
    CouldNotGetParent(PathBuf),
    UnknownDirectory(PathBuf),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;
        match self {
            Dispatch(error) => write!(f, "{}", error),
            Builtin(error) => write!(f, "{}", error),
            Executable(error) => write!(f, "{}", error),
            State(error) => write!(f, "{}", error),
            Path(error) => write!(f, "{}", error),
        }
    }
}

impl Display for DispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use DispatchError::*;
        match self {
            UnknownCommand(command_name) => {
                write!(f, "Command '{}' could not be found", command_name)
            }
            CommandNotExecutable(permission_code) => {
                write!(
                    f,
                    "File has permission code {:#o}, which disallows execution",
                    permission_code
                )
            }
            UnreadableExecutableMetadata(path) => {
                write!(
                    f,
                    "Executable metadata at '{}' could not be read",
                    path.display()
                )
            }
        }
    }
}

impl Display for BuiltinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use BuiltinError::*;
        match self {
            WrongArgCount(expected, actual) => {
                write!(
                    f,
                    "Expected {} {}, found {}",
                    expected,
                    match expected {
                        1 => "argument",
                        _ => "arguments",
                    },
                    actual
                )
            }
            InvalidArg(argument) => {
                write!(f, "Argument '{}' is invalid", argument)
            }
            InvalidValue(value) => write!(f, "Argument value '{}' is invalid", value),
            UnreadableFileType(path) => {
                write!(
                    f,
                    "Filetype at path '{}' could not be determined",
                    path.display()
                )
            }
            UnreadableFileName(path) => {
                write!(
                    f,
                    "Filename at path '{}' could not be determined",
                    path.display()
                )
            }
            UnreadableDirectory(path) => {
                write!(f, "Directory '{}' could not be read", path.display())
            }
            FailedToRun => write!(f, "Failed to run builtin"),
        }
    }
}

impl Display for ExecutableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ExecutableError::*;
        match self {
            PathNoLongerExists(path) => {
                write!(f, "Path '{}' no longer exists", path.display())
            }
            FailedToExecute(exit_code) => {
                write!(f, "Executable failed with exit code {}", exit_code)
            }
            CouldNotWait => write!(f, "Failed to wait for executable to complete"),
        }
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use StateError::*;
        match self {
            MissingEnv(variable) => {
                write!(
                    f,
                    "Environment variable '{}' missing from parent process",
                    variable
                )
            }
            CouldNotUpdateEnv(variable) => {
                write!(
                    f,
                    "Environment variable '{}' could not be updated",
                    variable
                )
            }
            NoPreviousDirectory => write!(f, "No previous directory"),
            NoNextDirectory => write!(f, "No next directory"),
            UnopenableConfig(path) => {
                write!(
                    f,
                    "Configuration file '{}' could not be openeed",
                    path.display()
                )
            }
            UnreadableConfig(path) => {
                write!(
                    f,
                    "Configuration file '{}' could not be read",
                    path.display()
                )
            }
            UnsupportedTerminal => write!(f, "Terminal is not supported"),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PathError::*;
        match self {
            FailedToConvertStringToPath(string) => {
                write!(f, "Failed to convert string '{}' to path", string)
            }
            FailedToConvertPathToString(path) => {
                // ? Under what circumstances would a path fail to convert but still display?
                write!(f, "Failed to convert path '{}' to string", path.display())
            }
            CouldNotCanonicalize(path) => {
                write!(f, "Path '{}' could not be canonicalized", path.display())
            }
            CouldNotGetParent(path) => {
                write!(
                    f,
                    "Parent directory of path '{}' could not be determined",
                    path.display()
                )
            }
            UnknownDirectory(path) => {
                write!(
                    f,
                    "Path '{}' does not exist or is inaccessible",
                    path.display()
                )
            }
        }
    }
}

/// Shortcut for creating a `RushError::Dispatch` without explicit imports
macro_rules! dispatch_err {
    ($content:expr) => {{
        use crate::errors::DispatchError::*;
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Dispatch($content))
    }};
}

/// Shortcut for creating a `RushError::Builtin` without explicit imports
macro_rules! builtin_err {
    ($content:expr) => {{
        use crate::errors::BuiltinError::*;
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Builtin($content))
    }};
}

/// Shortcut for creating a `RushError::Executable` without explicit imports
macro_rules! executable_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::ExecutableError::*;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Executable($content))
    }};
}

/// Shortcut for creating a `RushError::State` without explicit imports
macro_rules! state_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        use crate::errors::StateError::*;
        RushError::new(ErrorKind::State($content))
    }};
}

/// Shortcut for creating a `RushError::Path` without explicit imports
macro_rules! path_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::PathError::*;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Path($content))
    }};
}
