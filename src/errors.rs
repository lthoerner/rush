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
    Path(FileError),
}

/// Error type for errors which occur during command dispatch.
pub enum DispatchError {
    /// OVERVIEW
    /// This error occurs when a command cannot be resolved by the dispatcher.
    ///
    /// COMMON CAUSES
    /// - The command name was misspelled.
    /// - The command name was incorrect (e.g. 'vscode' instead of 'code')
    /// - The associated program is not installed.
    /// - The associated program is not in the PATH.
    /// - The associated program is only executable by superuser.
    ///
    /// RARE CAUSES
    /// - The associated program is installed but only available to a specific user.
    /// - The shell did not properly load the PATH environment variable.
    ///
    /// SOLUTIONS
    /// - Check the spelling of the command name.
    /// - Check the command name against the documentation of the associated program.
    /// - Ensure that the associated program is installed.
    /// - Ensure that the associated program is in the PATH.
    /// - Run the command using sudo.
    ///
    /// TECHNICAL DETAILS
    /// When attempting to resolve a command, the dispatcher will first look in its builtin command
    /// table, then scan all directories in the PATH environment variable for a file matching the
    /// provided command name. If no file is found, this error is returned.
    UnknownCommand(String),

    /// OVERVIEW
    /// This error occurs when the dispatcher locates a file matching the provided command name,
    /// but the file is not executable.
    ///
    /// CAUSE
    /// - The file permissions of the file do not allow it to be executed.
    ///
    /// SOLUTION
    /// - Add executable permissions to the file using a command such as 'chmod'.
    ///
    /// TECHNICAL DETAILS
    /// When attempting to resolve a command, the dispatcher will scan all directories in the PATH
    /// environment variable for a file matching the provided command name. If a file is found, the
    /// dispatcher then checks whether the file's permissions indicate that it is executable. If it
    /// is not, this error is returned.
    CommandNotExecutable(u32),

    /// OVERVIEW
    /// This error occurs when the dispatcher locates a file matching the provided command name,
    /// but the file's permission metadata cannot be read.
    ///
    /// CAUSE
    /// - The user does not have permission to read the file's metadata.
    ///
    /// SOLUTION
    /// - Ensure that the user has permission to read the file's metadata.
    ///
    /// TECHNICAL DETAILS
    /// When attempting to resolve a command, the dispatcher will scan all directories in the PATH
    /// environment variable for a file matching the provided command name. If a file is found, the
    /// dispatcher then checks whether the file's permissions indicate that it is executable. If it
    /// cannot, this error is returned.
    UnreadableExecutableMetadata(PathBuf),
}

/// Error type for errors that occur during the execution of builtin commands.
pub enum BuiltinError {
    /// OVERVIEW
    /// This error occurs when a builtin command is called with an incorrect number of arguments.
    ///
    /// CAUSE
    /// - The number of arguments supplied does not match the number of arguments expected.
    ///
    /// SOLUTION
    /// - Check the builtin's documentation and adjust arguments accordingly.
    ///
    /// TECHNICAL DETAILS
    /// When executing a builtin command, it checks the number of arguments provided by the user,
    /// and if it does not match the expected count, this error is returned in order to prevent the
    /// builtin from executing with malformed input.
    WrongArgCount(usize, usize),

    /// OVERVIEW
    /// This error occurs when a builtin command receives an argument it does not recognize.
    ///
    /// CAUSE
    /// - The builtin command is provided with an argument that it is unable to process.
    ///
    /// SOLUTION
    /// - Check the builtin's documentation and adjust arguments accordingly.
    ///
    /// TECHNICAL DETAILS
    /// When executing a builtin command, if it encounters an argument it does not recognize, this
    /// error is returned in order to prevent the builtin from executing with malformed input.
    InvalidArg(String),

    /// OVERVIEW
    /// This error occurs when a builtin command receives an invalid value for a valid argument.
    ///
    /// CAUSE
    /// - The builtin command receives a value that it cannot use for its associated argument.
    ///
    /// SOLUTION
    /// - Check the builtin's documentation and adjust arguments accordingly.
    ///
    /// TECHNICAL DETAILS
    /// When executing a builtin command, if it encounters a valid argument, it will typically try
    /// to parse the provided value for the argument. If the value is not expected by the parsing
    /// logic, this error is returned in order to prevent the builtin from executing with malformed
    /// input.
    InvalidValue(String),

    /// OVERVIEW
    /// This error occurs when a builtin is unable to interact with the terminal.
    ///
    /// COMMON CAUSES
    /// - The operation being performed is not supported by the terminal.
    /// - The terminal is not supported by Rush.
    ///
    /// RARE CAUSES
    /// - The stdout or stderr streams between the shell and the terminal have been corrupted.
    ///
    /// SOLUTIONS
    /// - Run the command in a different terminal.
    /// - Re-launch the terminal and run the command again.
    ///
    /// TECHNICAL DETAILS
    /// When executing a builtin command, it may perform operations on the terminal such as clearing
    /// the screen, moving the cursor around, or querying the terminal size. If for whatever reason
    /// it is unable to do so, this error is returned.
    TerminalOperationFailed,
}

/// Error type for errors which occur during execution of executable files.
pub enum ExecutableError {
    /// OVERVIEW
    /// This error occurs when an executable file is no longer accessible.
    ///
    /// CAUSE
    /// - The executable file was deleted or moved after being located by the dispatcher, but before
    /// being executed.
    ///
    /// SOLUTION
    /// - Ensure that the executable file is in a location that will not be modified without
    /// explicit user action.
    ///
    /// TECHNICAL DETAILS
    /// When dispatching an executable, the dispatcher will first locate the executable file in one
    /// of the directories in the PATH environment variable. If the file is found, it will then be
    /// set up with the appropriate environment variables and executed. If the file has been deleted
    /// or moved after being located but before being executed, this error is returned.
    PathNoLongerExists(PathBuf),

    /// OVERVIEW
    /// This error occurs when an executable returns a non-zero exit code.
    ///
    /// COMMON CAUSES
    /// - The arguments provided to the executable were invalid.
    /// - The executable was unable to locate a file it needed.
    /// - The executable was unable to complete its task for some other reason.
    ///
    /// RARE CAUSES
    /// - The executable has a bug which causes it to return the wrong exit code.
    /// - The executable uses non-conventional exit codes.
    /// - The executable was located but could not be executed (code 126).
    ///
    /// SOLUTIONS
    /// - Check the executable's documentation and adjust arguments accordingly.
    /// - Ensure that all needed files are accessible to the executable.
    /// - Reinstall the program associated with the executable.
    ///
    /// TECHNICAL DETAILS
    /// It is conventional for executables to return a zero exit code when they complete
    /// successfully, and a non-zero exit code when they fail. After running an executable, the
    /// disptacher will check its exit code, and if it is non-zero, this error is returned.
    FailedToExecute(isize),

    /// This error is exceedingly rare and its cause is unknown. It is not expected to occur.
    CouldNotWait,
}

/// Error type for errors which occur during state operations.
pub enum StateError {
    /// OVERVIEW
    /// This error occurs when an environment variable is missing from the parent process.
    ///
    /// COMMON CAUSES
    /// - The shell was launched from a non-standard environment.
    ///
    /// RARE CAUSES
    /// - The environment variable exists but is not accessible to the shell.
    ///
    /// SOLUTIONS
    /// - Ensure that the shell is launched from a standard environment.
    ///
    /// TECHNICAL DETAILS
    /// When the shell is launched, it copies the environment variables such as HOME and PATH from
    /// the parent process for its own use. If an environment variable is missing from the parent
    /// process, this error is returned.
    MissingEnv(EnvVariable),

    /// OVERVIEW
    /// This error occurs when an environment variable cannot be updated.
    ///
    /// COMMON CAUSES
    /// - The value provided to the environment variable was malformed due to an internal bug.
    /// - The value provided to the environment variable was malformed due to invalid input, such as
    /// a non-existent directory being provided to CWD.
    ///
    /// RARE CAUSES
    /// - The environment variable is not accessible to the shell.
    /// - The environment variable is not writable.
    ///
    /// SOLUTIONS
    /// - Ensure that no invalid values are provided to environment variables.
    /// - File an issue on the Rush repository if an internal bug is suspected.
    ///
    /// TECHNICAL DETAILS
    /// During execution, the shell keeps track of its environment through two mechanisms. Firstly,
    /// it has an internal representation of the environment variables, which are strongly-typed and
    /// validated. Secondly, it updates its process environment, which is a construct of the OS, to
    /// match its internal representation. This must be done in order for certain syscall-dependent
    /// function calls to be correctly performed. If the process environment cannot be updated, this
    /// error is returned.
    CouldNotUpdateEnv(EnvVariable),

    /// OVERVIEW
    /// This error occurs when the user erroneously invokes the 'previous-directory' builtin.
    ///
    /// CAUSE
    /// - The user ran the 'previous-directory' builtin, but the directory history is empty.
    ///
    /// SOLUTION
    /// - Navigate to another directory before returning to the previous one.
    ///
    /// TECHNICAL DETAILS
    /// The 'previous-directory' builtin allows the user to navigate to the directory they were in
    /// before navigating to the current one. This is implemented using a stack of directories, very
    /// similarly to how browser tabs handle navigation. If the user attempts to navigate to the
    /// previous directory when the stack is empty, this error is returned.
    NoPreviousDirectory,

    /// OVERVIEW
    /// This error occurs when the user erroneously invokes the 'next-directory' builtin.
    ///
    /// CAUSE
    /// - The user ran the 'next-directory' builtin, but the directory history is empty.
    ///
    /// SOLUTION
    /// - Navigate to another directory before returning to the next one.
    ///
    /// TECHNICAL DETAILS
    /// The 'next-directory' builtin allows the user to navigate to the directory they were in after
    /// navigating to the current one. This is implemented using a stack of directories, very
    /// similarly to how browser tabs handle navigation. If the user attempts to navigate to the
    /// next directory when the stack is empty, this error is returned.
    NoNextDirectory,

    /// OVERVIEW
    /// This error occurs when the line editor is unable to interact with the terminal.
    ///
    /// CAUSE
    /// - The terminal is not supported by Rustyline (the line editor library).
    ///
    /// SOLUTION
    /// - Run the shell in a different terminal.
    ///
    /// TECHNICAL DETAILS
    /// The line editor is responsible for handling user input and displaying the prompt. It is
    /// implemented using the Rustyline library, which should support most terminals. If the
    /// terminal being used does not support the requisite features, this error is returned.
    UnsupportedTerminal,
}

/// Error type for errors which occur during path operations.
pub enum FileError {
    /// OVERVIEW
    /// This error occurs when the shell is unable to convert a string to a path.
    ///
    /// CAUSE
    /// - One or more of the paths in the PATH environment variable are invalid.
    ///
    /// SOLUTION
    /// - Make sure all paths in the PATH variable exist and are accessible.
    ///
    /// TECHNICAL DETAILS
    /// When the shell is launched, it copies the PATH environment variable from the parent process
    /// and converts it to an internal representation for its own use. Because of how environment
    /// variables are largely just unvalidated strings, it is possible for the PATH to contain
    /// invalid paths. If any of the paths scannot be converted, this error is returned.
    FailedToConvertStringToPath(String),

    /// OVERVIEW
    /// - This error occurs when the shell is unable to convert a path to a string.
    ///
    /// CAUSE
    /// - The shell is attempting to display or operate on a path which is not valid UTF-8.
    ///
    /// SOLUTION
    /// - File an issue on the Rush repository.
    ///
    /// TECHNICAL DETAILS
    /// When the shell is attempting to display a path, or performing operations to manipulate the
    /// path which require it to be a string, it may be unable to convert the path to a string, at
    /// which point it returns this error. This is largely because, even though a path may be valid
    /// on the filesystem, the OS and filesystem may not use UTF-8 encoding.
    FailedToConvertPathToString(PathBuf),

    /// OVERVIEW
    /// This error occurs when the shell is unable to canonicalize a path.
    ///
    /// CAUSES
    /// - The path does not exist or is inaccessible.
    /// - The path is misspelled or otherwise malformed.
    ///
    /// SOLUTION
    /// - Ensure that the path exists, is accessible by the user, and is formatted correctly.
    ///
    /// TECHNICAL DETAILS
    /// When the shell receives a path as input, it will attempt to canonicalize it, which is to say
    /// it will attempt to resolve any relative paths, symbolic links, and shorthands such as '~'.
    /// If the absolute path cannot be determined, this error is returned.
    CouldNotCanonicalize(PathBuf),

    /// This error is exceedingly rare and its cause is unknown. It is not expected to occur.
    CouldNotGetParent(PathBuf),

    /// OVERVIEW
    /// This error occurs when a file cannot be opened.
    ///
    /// COMMON CAUSES
    /// - The file does not exist or is inaccessible.
    /// - The file is a directory.
    ///
    /// RARE CAUSES
    /// - The file is a symbolic link.
    /// - The file is a special file, such as a device file.
    ///
    /// SOLUTIONS
    /// - Ensure that the file exists, is accessible by the user, and is not a directory any other
    /// special file type.
    CouldNotOpenFile(PathBuf),

    /// OVERVIEW
    /// This error occurs when a file's contents are invalid or inaccessible.
    ///
    /// COMMON CAUSES
    /// - The file being read is the config file, and its contents are malformed.
    ///
    /// RARE CAUSES
    /// - The file was modified by another program while the shell was reading it.
    /// - The file has been corrupted.
    ///
    /// SOLUTIONS
    /// - If the file in question is a config file, make sure it is formatted correctly.
    /// - Ensure that the file is not open in or being modified by another program. This is usually
    /// guaranteed by the OS/filesystem.
    ///
    /// TECHNICAL DETAILS
    /// When the shell is reading a configuration file, it will attempt to parse its contents based
    /// on the expected format for the config. If the contents are invalid, this error is returned.
    /// This error may also be returned upon reading any file if the file is somehow externally
    /// modified in such a way that the reading process is interrupted.
    CouldNotReadFile(PathBuf),

    /// OVERVIEW
    /// This error occurs when a file cannot be created.
    ///
    /// COMMON CAUSES
    /// - The file already exists.
    /// - The file's enclosing directory does not exist or is invalid.
    /// - The file's enclosing directory is not writable.
    ///
    /// RARE CAUSES
    /// - The disk is full.
    /// - The filesystem is read-only.
    ///
    /// SOLUTIONS
    /// - Check that the file does not already exist.
    /// - Ensure that the file's enclosing directory exists and is writable.
    CouldNotCreateFile(PathBuf),

    /// OVERVIEW
    /// This error occurs when a file cannot be deleted.
    ///
    /// COMMON CAUSES
    /// - The file does not exist.
    /// - The file is not writable.
    /// - The file is open in another program.
    ///
    /// RARE CAUSES
    /// - The file is a special file, such as a device file.
    /// - The filesystem is read-only.
    ///
    /// SOLUTIONS
    /// - Check that the file exists and is not a special file.
    /// - Check that the file is not open in another program.
    /// - Ensure that the file is writable.
    CouldNotDeleteFile(PathBuf),

    /// OVERVIEW
    /// This error occurs when a directory cannot be created.
    ///
    /// COMMON CAUSES
    /// - The directory already exists.
    /// - The directory's parent directory does not exist or is invalid.
    /// - The directory's parent directory is not writable.
    ///
    /// RARE CAUSES
    /// - The disk is full.
    /// - The filesystem is read-only.
    ///
    /// SOLUTIONS
    /// - Check that the directory does not already exist.
    /// - Ensure that the directory's parent directory exists and is writable.
    CouldNotCreateDirectory(PathBuf),

    /// OVERVIEW
    /// This error occurs when the shell is unable to determine the type of a file.
    ///
    /// CAUSE
    /// - The file does not exist or is inaccessible.
    ///
    /// SOLUTION
    /// - Ensure that the file exists and is accessible by the user.
    ///
    /// TECHNICAL DETAILS
    /// The "type" of a file refers to its filesystem classification. For example, a file may be a
    /// regular file, a directory, a symbolic link, or a device file. When the shell needs to
    /// determine the type of a file, it will attempt to read its metadata. If the relevant metadata
    /// cannot be read, this error is returned.
    UnreadableFileType(PathBuf),

    /// This error is exceedingly rare and its cause is unknown. It is not expected to occur.
    UnreadableFileName(PathBuf),

    /// OVERVIEW
    /// This error occurs when the shell is unable to read the contents of a directory.
    ///
    /// CAUSE
    /// - The directory does not exist or is inaccessible.
    ///
    /// SOLUTION
    /// - Ensure that the directory exists and is accessible by the user.
    UnreadableDirectory(PathBuf),

    /// OVERVIEW
    /// This error occurs when the shell is unable to locate a file or directory.
    ///
    /// CAUSE
    /// - The file or directory does not exist or is inaccessible.
    ///
    /// SOLUTION
    /// - Ensure that the file or directory exists and is accessible by the user.
    UnknownPath(PathBuf),
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
            UnsupportedTerminal => write!(f, "Terminal is not supported"),
        }
    }
}

impl Display for FileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FileError::*;
        match self {
            FailedToConvertStringToPath(string) => {
                write!(f, "Failed to convert string '{}' to path", string)
            }
            FailedToConvertPathToString(path) => {
                // ? Under what circumstances would a path fail to convert but still display?
                write!(f, "Failed to convert path '{}' to string", path.display())
            }
            CouldNotCanonicalize(path) => {
                write!(f, "Could not canonicalize path '{}'", path.display())
            }
            CouldNotGetParent(path) => {
                write!(
                    f,
                    "Could not determine parent directory of path '{}'",
                    path.display()
                )
            }
            CouldNotOpenFile(path) => {
                write!(f, "Could not open file at path '{}'", path.display())
            }
            CouldNotReadFile(path) => {
                write!(f, "Could not read file at path '{}'", path.display())
            }
            CouldNotCreateFile(path) => {
                write!(f, "Could not create file at path '{}'", path.display())
            }
            CouldNotDeleteFile(path) => {
                write!(f, "Could not delete file at path '{}'", path.display())
            }
            CouldNotCreateDirectory(path) => {
                write!(f, "Could not create directory at path '{}'", path.display())
            }
            UnreadableFileType(path) => {
                write!(
                    f,
                    "Could not determine file type of path '{}'",
                    path.display()
                )
            }
            UnreadableFileName(path) => {
                write!(
                    f,
                    "Could not determine file name of path '{}'",
                    path.display()
                )
            }
            UnreadableDirectory(path) => {
                write!(f, "Could not read directory at path '{}'", path.display())
            }
            UnknownPath(path) => {
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
macro_rules! file_err {
    ($content:expr) => {{
        use crate::errors::ErrorKind;
        use crate::errors::FileError::*;
        use crate::errors::RushError;
        RushError::new(ErrorKind::Path($content))
    }};
}
