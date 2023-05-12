use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fmt, io};

use anyhow::anyhow;
use thiserror::Error;

/// This is a wrapper for io::Error to add more context than the default Display.
/// It should not be used directly. Use an internal error instead.
#[derive(Error, Debug)]
pub struct IoError {
    #[from]
    source: io::Error,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source)?;
        if let Some(e) = self.source.get_ref() {
            if let Some(e) = e.source() {
                write!(f, " because: {}", e)?;
            }
        }
        Ok(())
    }
}

pub enum FileContext {
    FileRead,
}

pub enum ProcessContext {
    WaitingForChild,
}

/// This is for creating errors with additional context, e.g.
/// ```ignore
/// read(file).file_context(FileContext::FileRead)?;
/// ```
/// General idea: we can get more reuse by making a general conversion from io::Error + context -> descriptive error
pub trait IoErrorContextExt {
    type T;
    fn file_context(self, ctx: FileContext) -> anyhow::Result<Self::T>;
    fn process_context(self, ctx: ProcessContext) -> anyhow::Result<Self::T>;
}

impl<T> IoErrorContextExt for Result<T, io::Error> {
    type T = T;
    fn file_context(self, ctx: FileContext) -> anyhow::Result<T> {
        self.map_err(|e| {
            match (e.kind(), ctx) {
                (ErrorKind::NotFound, FileContext::FileRead) => anyhow!("File not found"),
                (ErrorKind::PermissionDenied, FileContext::FileRead) => {
                    anyhow!("No permission to read file")
                }
                _ => IoError::from(e).into(),
            }
        })
    }

    fn process_context(self, ctx: ProcessContext) -> anyhow::Result<Self::T> {
        self.map_err(|e| {
            // ctx can currently only be WaitingForChild,
            // which can only fail with ECHILD (see waitpid)
            // which Rust doesn't expose
            #[allow(clippy::match_single_binding)]
            match (e.kind(), ctx) {
                _ => IoError::from(e).into(),
            }
        })
    }
}

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
    #[error("Unable to read Path: {0}")]
    FailedReadingPath(PathBuf),
    #[error("Unable to read file type from path: {0}")]
    FailedReadingFileType(PathBuf),
    #[error("Unable to read file name from path: {0}")]
    FailedReadingFileName(PathBuf),
    #[error("Unable to read dir: {0}")]
    FailedReadingDir(PathBuf),
}

#[derive(Error, Debug)]
pub enum ExecutableError {
    #[error("Path no longer exists: {0}")]
    PathNoLongerExists(PathBuf),
    #[error("Executable failed with exit code: {0}")]
    FailedToExecute(isize),
    #[error("Failed to parse executable stdout: {0}")]
    FailedToParseStdout(String),
    #[error("Failed to parse executable stderr: {0}")]
    FailedToParseStderr(String),
}
