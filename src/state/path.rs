use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::path::{Path as StdPath, PathBuf};

use fs_err::canonicalize;

use crate::errors::{Handle, Result};

/// Wrapper class for a `PathBuf`
/// Adds convenience methods for displaying the path in a user-friendly way,
/// along with guarantees about path validity that are not provided by `PathBuf`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    absolute_path: PathBuf,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.absolute_path.display())
    }
}

impl From<Path> for PathBuf {
    fn from(value: Path) -> Self {
        value.absolute_path
    }
}

impl Path {
    /// Attempts to construct a new `Path` from a string by resolving it to an absolute path
    pub fn try_from_str(path: &str, home_directory: &StdPath) -> Result<Self> {
        // The home directory shorthand must be expanded before resolving the path,
        // because PathBuf is not user-aware and only uses absolute and relative paths
        let expanded_path = expand_home(path, home_directory)?;
        // Canonicalizing a path will resolve any relative or absolute paths
        let absolute_path = canonicalize(&expanded_path)
            .replace_err(|| file_err!(CouldNotCanonicalize: expanded_path))?;

        // If the file system can canonicalize the path, it should exist,
        // but this is added for extra precaution
        if !absolute_path.exists() {
            Err(file_err!(UnknownPath: absolute_path))
        } else {
            Ok(Self { absolute_path })
        }
    }

    /// Attempts to locate an executable file in the PATH
    // ? Should this be a method of `Environment` instead?
    pub fn try_resolve_executable(name: &str, path: &VecDeque<Path>) -> Result<Self> {
        if !name.is_empty() {
            for dir in path {
                let mut path = dir.path().clone();
                path.push(name);

                if path.exists() {
                    return Ok(Self {
                        absolute_path: path,
                    });
                }
            }
        }

        Err(file_err!(CouldNotCanonicalize: name))
    }

    /// Returns the absolute path
    pub fn path(&self) -> &PathBuf {
        &self.absolute_path
    }

    /// Gets the shortened version of the path
    /// If a truncation factor is provided, the path will be truncated
    /// The shortened path will always have the home directory collapsed
    pub fn collapse(&self, home_directory: &PathBuf, truncation_factor: Option<usize>) -> String {
        // ? Is there a less redundant way to write this?
        let path = match self.absolute_path.strip_prefix(home_directory) {
            Ok(path) => {
                let mut path_string = path.to_string_lossy().to_string();
                // ? Is this really necessary? Wouldn't it be fine to just have '~/'?
                path_string = match path_string.len() {
                    0 => String::from("~"),
                    _ => format!("~/{}", path_string),
                };

                path_string
            }
            Err(_) => self.to_string(),
        };

        // $ This might cause a bug with non-unicode characters (paths use OsString which is not guaranteed to be valid unicode)
        let directories: Vec<String> = path.split('/').map(|d| d.to_string()).collect();
        let mut truncated_directories = Vec::new();

        if let Some(factor) = truncation_factor {
            for dir in directories {
                let mut truncated_dir = dir.clone();
                if dir.len() > factor {
                    truncated_dir.truncate(factor);
                }

                truncated_directories.push(truncated_dir);
            }
        } else {
            truncated_directories = directories;
        }

        truncated_directories.join("/")
    }
}

/// Expands the home directory shorthand in a path string
fn expand_home(path: &str, home_directory: &StdPath) -> Result<PathBuf> {
    if path.starts_with('~') {
        Ok(PathBuf::from(
            path.replace(
                '~',
                home_directory
                    .to_str()
                    .replace_err(|| file_err!(FailedToConvertPathToString: home_directory))?,
            ),
        ))
    } else {
        Ok(PathBuf::from(path))
    }
}
