use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::state::EnvVariable;

pub type Result<T> = std::result::Result<T, RushError>;
pub trait Handle<T> {
    /// Replaces any error kind with a new one, with additional context
    fn replace_err(self, new_error: RushError, context: &str) -> Result<T>;
    /// Replaces any error kind with a new one, without additional context
    fn replace_err_no_context(self, new_error: RushError) -> Result<T>;
}

impl<T, E> Handle<T> for std::result::Result<T, E> {
    fn replace_err(mut self, new_error: RushError, context: &str) -> Result<T> {
        self.map_err(|_| new_error.set_context(context))
    }

    fn replace_err_no_context(mut self, new_error: RushError) -> Result<T> {
        self.map_err(|_| new_error)
    }
}

impl<T> Handle<T> for std::option::Option<T> {
    fn replace_err(mut self, new_error: RushError, context: &str) -> Result<T> {
        self.ok_or(new_error.set_context(context))
    }

    fn replace_err_no_context(mut self, new_error: RushError) -> Result<T> {
        self.ok_or(new_error)
    }
}

pub struct RushError {
    kind: ErrorKind,
    context: Option<String>,
}

impl Display for RushError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(context) = &self.context {
            write!(f, "{}: {}", self.kind, context)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

impl RushError {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            context: None,
        }
    }

    pub fn set_context(mut self, context: &str) -> Self {
        self.context = Some(context.to_owned());
        self
    }
}

pub enum ErrorKind {
    Dispatch(DispatchError),
    Builtin(BuiltinError),
    Executable(ExecutableError),
    State(StateError),
    Path(PathError),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Dispatch(error) => write!(f, "{}", error),
            ErrorKind::Builtin(error) => write!(f, "{}", error),
            ErrorKind::Executable(error) => write!(f, "{}", error),
            ErrorKind::State(error) => write!(f, "{}", error),
            ErrorKind::Path(error) => write!(f, "{}", error),
        }
    }
}

pub enum DispatchError {
    UnknownCommand(String),
    CommandNotExecutable(u32),
    FailedToReadExecutableMetadata(PathBuf),
}

impl Display for DispatchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchError::UnknownCommand(command_name) => {
                write!(f, "Unknown command: {}", command_name)
            }
            DispatchError::CommandNotExecutable(permission_code) => {
                write!(
                    f,
                    "Command is not executable. Permission code: {}",
                    permission_code
                )
            }
            DispatchError::FailedToReadExecutableMetadata(path) => {
                write!(
                    f,
                    "Failed to read metadata for executable: {}",
                    path.display()
                )
            }
        }
    }
}

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

impl Display for BuiltinError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuiltinError::InvalidArgumentCount(count) => {
                write!(f, "Incorrect number of arguments: {}", count)
            }
            BuiltinError::InvalidArgument(argument) => {
                write!(f, "Invalid argument: {}", argument)
            }
            BuiltinError::InvalidValue(value) => write!(f, "Invalid argument value: {}", value),
            BuiltinError::FailedToRun => write!(f, "Failed to run builtin"),
            BuiltinError::FailedReadingPath(path) => {
                write!(f, "Failed to read path: {}", path.display())
            }
            BuiltinError::FailedReadingFileType(path) => {
                write!(f, "Failed to read file type from path: {}", path.display())
            }
            BuiltinError::FailedReadingFileName(path) => {
                write!(f, "Failed to read file name from path: {}", path.display())
            }
            BuiltinError::FailedReadingDir(path) => {
                write!(f, "Failed to read dir: {}", path.display())
            }
        }
    }
}

pub enum ExecutableError {
    PathNoLongerExists(PathBuf),
    FailedToExecute(isize),
}

impl Display for ExecutableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutableError::PathNoLongerExists(path) => {
                write!(f, "Path no longer exists: {}", path.display())
            }
            ExecutableError::FailedToExecute(exit_code) => {
                write!(f, "Executable failed with exit code: {}", exit_code)
            }
        }
    }
}

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

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::MissingExternalEnvironmentVariable(variable) => {
                write!(f, "Missing external environment variable: {}", variable)
            }
            StateError::MissingInternalEnvironmentVariable(variable) => {
                write!(f, "Missing internal environment variable: {}", variable)
            }
            StateError::FailedToUpdateEnvironmentVariable(variable) => {
                write!(f, "Failed to update environment variable: {}", variable)
            }
            StateError::NoPreviousDirectory => write!(f, "No previous directory"),
            StateError::NoNextDirectory => write!(f, "No next directory"),
            StateError::FailedToOpenConfigFile(path) => {
                write!(f, "Failed to open configuration file: {}", path.display())
            }
            StateError::FailedToReadConfigFile(path) => {
                write!(f, "Failed to read configuration file: {}", path.display())
            }
            StateError::Uncategorized => write!(f, "Unknown error"),
        }
    }
}

pub enum PathError {
    FailedToConvertPathBufToString(PathBuf),
    FailedToCanonicalize(PathBuf),
    FailedToAccess(PathBuf),
    UnknownDirectory(PathBuf),
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::FailedToConvertPathBufToString(path) => {
                write!(f, "Failed to convert PathBuf to String: {}", path.display())
            }
            PathError::FailedToCanonicalize(path) => {
                write!(
                    f,
                    "Failed to canonicalize directory path: {}",
                    path.display()
                )
            }
            PathError::FailedToAccess(path) => {
                write!(f, "Failed to access directory path: {}", path.display())
            }
            PathError::UnknownDirectory(path) => {
                write!(f, "Directory does not exist: {}", path.display())
            }
        }
    }
}

macro_rules! dispatch_err {
    ($content:expr) => {{
        use crate::errors::DispatchError::*;
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Dispatch($content))
    }};
}

macro_rules! builtin_err {
    ($content:expr) => {{
        use crate::errors::BuiltinError::*;
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Builtin($content))
    }};
}

macro_rules! executable_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::ExecutableError::*;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Executable($content))
    }};
}

macro_rules! state_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::RushError;
        use crate::errors::StateError::*;
        RushError::new(ErrorKind::State($content))
    }};
}

macro_rules! path_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::PathError::*;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Path($content))
    }};
}
