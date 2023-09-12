use std::collections::{HashMap, VecDeque};
use std::env;
use std::fmt::{Display, Formatter};
use std::path::{Path as StdPath, PathBuf};

use bitflags::bitflags;
use clap::ValueEnum;

use super::path::Path;
use crate::errors::{Handle, Result};

/// Identifier enum for safely accessing environment variables
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
pub enum EnvVariable {
    USER,
    HOME,
    CWD,
    PATH,
}

impl Display for EnvVariable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::USER => "USER",
                Self::HOME => "HOME",
                Self::CWD => "CWD",
                Self::PATH => "PATH",
            }
        )
    }
}

impl EnvVariable {
    /// Does the same thing as `.to_string()`, but uses legacy environment variable names
    fn to_legacy_string(self) -> String {
        match self {
            Self::USER => "USER".to_owned(),
            Self::HOME => "HOME".to_owned(),
            Self::CWD => "PWD".to_owned(),
            Self::PATH => "PATH".to_owned(),
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
        let USER = get_parent_env_var(EnvVariable::USER)?;
        let HOME = PathBuf::from(get_parent_env_var(EnvVariable::HOME)?);
        let CWD = Path::try_from_str(get_parent_env_var(EnvVariable::CWD)?.as_str(), Some(&HOME))?;
        let PATH = convert_path_var(get_parent_env_var(EnvVariable::PATH)?.as_str())?;

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
                .replace_err(|| state_err!(CouldNotUpdateEnv: EnvVariable::CWD))?;
        }

        Ok(())
    }

    /// Sets the current working directory and stores the previous working directory
    pub fn set_CWD(&mut self, new_directory: &StdPath, history_limit: Option<usize>) -> Result<()> {
        let starting_directory = self.CWD.clone();
        let new_directory = Path::try_from_path(new_directory, Some(&self.HOME))?;

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
    std::env::var(variable.to_legacy_string()).replace_err(|| state_err!(MissingEnv: variable))
}

/// Converts the PATH environment variable from a string to a collection of `Path`s
fn convert_path_var(path: &str) -> Result<VecDeque<Path>> {
    let mut paths = VecDeque::new();
    let path_strings = path.split(':').collect::<Vec<&str>>();

    for path_string in path_strings {
        if let Ok(path) = Path::try_from_str(path_string, None) {
            paths.push_back(path);
        }
    }

    Ok(paths)
}
