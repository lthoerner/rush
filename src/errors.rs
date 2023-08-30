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
    FailedToReadExecutableMetadata(PathBuf),
}

/// Error type for errors which occur during execution of builtins.
pub enum BuiltinError {
    InvalidArgumentCount(usize),
    InvalidArgument(String),
    InvalidValue(String),
    // TODO: Break this into multiple error types
    FailedToRun,
    FailedReadingPath(PathBuf),
    FailedReadingFileType(PathBuf),
    FailedReadingFileName(PathBuf),
    FailedReadingDir(PathBuf),
}

/// Error type for errors which occur during execution of executable files.
pub enum ExecutableError {
    PathNoLongerExists(PathBuf),
    FailedToExecute(isize),
}

/// Error type for errors which occur during state operations.
pub enum StateError {
    MissingExternalEnvironmentVariable(EnvVariable),
    MissingInternalEnvironmentVariable(EnvVariable),
    FailedToUpdateEnvironmentVariable(EnvVariable),
    NoPreviousDirectory,
    NoNextDirectory,
    FailedToOpenConfigFile(PathBuf),
    FailedToReadConfigFile(PathBuf),
    Uncategorized,
}

/// Error type for errors which occur during path operations.
pub enum PathError {
    FailedToConvertPathBufToString(PathBuf),
    FailedToCanonicalize(PathBuf),
    FailedToAccess(PathBuf),
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
                write!(f, "Unknown command: {}", command_name)
            }
            CommandNotExecutable(permission_code) => {
                write!(
                    f,
                    "Command is not executable. Permission code: {}",
                    permission_code
                )
            }
            FailedToReadExecutableMetadata(path) => {
                write!(
                    f,
                    "Failed to read metadata for executable: {}",
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
            InvalidArgumentCount(count) => {
                write!(f, "Incorrect number of arguments: {}", count)
            }
            InvalidArgument(argument) => {
                write!(f, "Invalid argument: {}", argument)
            }
            InvalidValue(value) => write!(f, "Invalid argument value: {}", value),
            FailedToRun => write!(f, "Failed to run builtin"),
            FailedReadingPath(path) => {
                write!(f, "Failed to read path: {}", path.display())
            }
            FailedReadingFileType(path) => {
                write!(f, "Failed to read file type from path: {}", path.display())
            }
            FailedReadingFileName(path) => {
                write!(f, "Failed to read file name from path: {}", path.display())
            }
            FailedReadingDir(path) => {
                write!(f, "Failed to read dir: {}", path.display())
            }
        }
    }
}

impl Display for ExecutableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ExecutableError::*;
        match self {
            PathNoLongerExists(path) => {
                write!(f, "Path no longer exists: {}", path.display())
            }
            FailedToExecute(exit_code) => {
                write!(f, "Executable failed with exit code: {}", exit_code)
            }
        }
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use StateError::*;
        match self {
            MissingExternalEnvironmentVariable(variable) => {
                write!(f, "Missing external environment variable: {}", variable)
            }
            MissingInternalEnvironmentVariable(variable) => {
                write!(f, "Missing internal environment variable: {}", variable)
            }
            FailedToUpdateEnvironmentVariable(variable) => {
                write!(f, "Failed to update environment variable: {}", variable)
            }
            NoPreviousDirectory => write!(f, "No previous directory"),
            NoNextDirectory => write!(f, "No next directory"),
            FailedToOpenConfigFile(path) => {
                write!(f, "Failed to open configuration file: {}", path.display())
            }
            FailedToReadConfigFile(path) => {
                write!(f, "Failed to read configuration file: {}", path.display())
            }
            Uncategorized => write!(f, "Unknown error"),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use PathError::*;
        match self {
            FailedToConvertPathBufToString(path) => {
                write!(f, "Failed to convert PathBuf to String: {}", path.display())
            }
            FailedToCanonicalize(path) => {
                write!(
                    f,
                    "Failed to canonicalize directory path: {}",
                    path.display()
                )
            }
            FailedToAccess(path) => {
                write!(f, "Failed to access directory path: {}", path.display())
            }
            UnknownDirectory(path) => {
                write!(f, "Directory does not exist: {}", path.display())
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
