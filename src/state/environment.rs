use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use bitflags::bitflags;

use super::path::Path;
use crate::errors::{Handle, Result};

/// Identifier enum for safely accessing environment variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnvVariable {
    User,
    Home,
    Cwd,
    Path,
}

impl Display for EnvVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::User => "USER",
                Self::Home => "HOME",
                Self::Cwd => "CWD",
                Self::Path => "PATH",
            }
        )
    }
}

impl EnvVariable {
    /// Does the same thing as `.to_string()`, but uses legacy environment variable names
    fn to_legacy_string(self) -> String {
        match self {
            Self::User => "USER".to_string(),
            Self::Home => "HOME".to_string(),
            Self::Cwd => "PWD".to_string(),
            Self::Path => "PATH".to_string(),
        }
    }
}

bitflags! {
    /// Flags for updating multiple environment variables at once
    pub struct EnvVariables: u8 {
        const USER = 0b0001;
        const HOME = 0b0010;
        const CWD = 0b0100;
        const PATH = 0b1000;
    }
}

/// Represents the shell environment by encapsulating the environment variables
// * Environment variables are represented in all caps by convention,
// * any fields that are not actual environment variables are represented in the usual snake_case
#[allow(non_snake_case)]
pub struct Environment {
    pub USER: String,
    pub HOME: PathBuf,
    pub CWD: Path,
    // * PATH is not to be confused with the WORKING_DIRECTORY. PATH is a list of directories which
    // * the shell will search for executables in. WORKING_DIRECTORY is the current directory the user is in.
    pub PATH: VecDeque<Path>,
    backward_directories: VecDeque<Path>,
    forward_directories: VecDeque<Path>,
    #[allow(dead_code)]
    custom_variables: HashMap<String, String>,
}

#[allow(non_snake_case)]
impl Environment {
    pub fn new() -> Result<Self> {
        let USER = get_parent_env_var(EnvVariable::User)?;
        let HOME = PathBuf::from(get_parent_env_var(EnvVariable::Home)?);
        let CWD = Path::try_from_str(get_parent_env_var(EnvVariable::Cwd)?.as_str(), &HOME)?;
        let PATH = convert_path(get_parent_env_var(EnvVariable::Path)?.as_str(), &HOME)?;

        Ok(Self {
            USER,
            HOME,
            CWD,
            PATH,
            backward_directories: VecDeque::new(),
            forward_directories: VecDeque::new(),
            custom_variables: HashMap::new(),
        })
    }

    /// Updates the shell process's environment variables to match the internal representation
    fn update_process_env_vars(&self, vars: EnvVariables) -> Result<()> {
        // TODO: How to detect errors here?
        if vars.contains(EnvVariables::USER) {
            env::set_var("USER", &self.USER);
        }

        if vars.contains(EnvVariables::HOME) {
            env::set_var("HOME", &self.HOME);
        }

        if vars.contains(EnvVariables::CWD) {
            env::set_current_dir(self.CWD.path())
                .replace_err(state_err!(CouldNotUpdateEnv(EnvVariable::Cwd)))?;
        }

        Ok(())
    }

    /// Sets the current working directory and stores the previous working directory
    pub fn set_CWD(&mut self, new_directory: &str, history_limit: Option<usize>) -> Result<()> {
        let starting_directory = self.CWD.clone();
        let new_directory = Path::try_from_str(new_directory, &self.HOME)?;

        // Add the old directory to the history, avoiding duplicates
        if new_directory != starting_directory {
            self.CWD = new_directory;
            self.backward_directories.push_back(starting_directory);
            self.forward_directories.clear();

            if let Some(limit) = history_limit {
                while self.backward_directories.len() > limit {
                    self.backward_directories.pop_front();
                }
            }

            self.update_process_env_vars(EnvVariables::CWD)?;
        }

        Ok(())
    }

    /// Sets the CWD to the previous working directory
    pub fn previous_directory(&mut self) -> Result<()> {
        let starting_directory = self.CWD.clone();
        if let Some(previous_path) = self.backward_directories.pop_back() {
            self.CWD = previous_path;
            self.forward_directories.push_front(starting_directory);
            self.update_process_env_vars(EnvVariables::CWD)
        } else {
            Err(state_err!(NoPreviousDirectory))
        }
    }

    /// Sets the CWD to the next working directory
    pub fn next_directory(&mut self) -> Result<()> {
        let starting_directory = self.CWD.clone();
        if let Some(next_path) = self.forward_directories.pop_front() {
            self.CWD = next_path;
            self.backward_directories.push_back(starting_directory);
            self.update_process_env_vars(EnvVariables::CWD)
        } else {
            Err(state_err!(NoNextDirectory))
        }
    }
}

/// Gets the environment variables from the parent process during shell initialization
fn get_parent_env_var(variable: EnvVariable) -> Result<String> {
    std::env::var(variable.to_legacy_string()).replace_err(state_err!(MissingEnv(variable)))
}

/// Converts the PATH environment variable from a string to a collection of `Path`s
fn convert_path(path: &str, home: &PathBuf) -> Result<VecDeque<Path>> {
    let mut paths = VecDeque::new();

    let path_strings = path.split(':').collect::<Vec<&str>>();
    for path_string in path_strings {
        let path = Path::try_from_str(path_string, home).replace_err(file_err!(
            FailedToConvertStringToPath(path_string.to_owned())
        ))?;
        paths.push_back(path);
    }

    Ok(paths)
}
