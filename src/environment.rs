use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

use anyhow::Result;

use crate::errors::ShellError;
use crate::path::Path;

// Represents the shell environment by encapsulating the environment variables
// * Environment variables are represented in all caps by convention,
// * any fields that are not actual environment variables are represented in the usual snake_case
#[allow(dead_code, non_snake_case)]
pub struct Environment {
    USER: String,
    HOME: PathBuf,
    pub WORKING_DIRECTORY: Path,
    backward_directories: VecDeque<Path>,
    forward_directories: VecDeque<Path>,
    // * PATH is not to be confused with the WORKING_DIRECTORY. PATH is a list of directories which
    // * the shell will search for executables in. WORKING_DIRECTORY is the current directory the user is in.
    PATH: VecDeque<Path>,
    custom_variables: HashMap<String, String>,
}

#[allow(non_snake_case)]
impl Environment {
    pub fn new() -> Result<Self> {
        let USER = get_parent_env_var("USER")?;
        let HOME = PathBuf::from(get_parent_env_var("HOME")?);
        let WORKING_DIRECTORY = Path::from_str(get_parent_env_var("PWD")?.as_str(), &HOME)?;
        let PATH = convert_path(get_parent_env_var("PATH")?.as_str(), &HOME)?;

        Ok(Self {
            USER,
            HOME,
            WORKING_DIRECTORY,
            backward_directories: VecDeque::new(),
            forward_directories: VecDeque::new(),
            PATH,
            custom_variables: HashMap::new(),
        })
    }

    // Updates the shell process's environment variables to match the internal representation
    fn update_process_env_vars(
        &self,
        set_user: bool,
        set_home: bool,
        set_working_directory: bool,
    ) -> Result<()> {
        if set_user {
            std::env::set_var("USER", &self.USER);
        }

        if set_home {
            std::env::set_var("HOME", &self.HOME);
        }

        if set_working_directory {
            std::env::set_current_dir(self.WORKING_DIRECTORY.path())
                .map_err(|_| ShellError::FailedToUpdateEnvironmentVariables)?;
        }

        Ok(())
    }

    pub fn USER(&self) -> &String {
        &self.USER
    }

    pub fn HOME(&self) -> &PathBuf {
        &self.HOME
    }

    pub fn PATH(&self) -> &VecDeque<Path> {
        &self.PATH
    }

    pub fn PATH_mut(&mut self) -> &mut VecDeque<Path> {
        &mut self.PATH
    }

    // Sets the current working directory and stores the previous working directory
    pub fn set_cwd(&mut self, new_path: &str) -> Result<()> {
        let previous_path = self.WORKING_DIRECTORY.clone();
        self.WORKING_DIRECTORY = Path::from_str(new_path, &self.HOME)?;
        self.backward_directories.push_back(previous_path);
        self.forward_directories.clear();
        self.update_process_env_vars(false, false, true)
    }

    // Sets the current working directory to the previous working directory
    pub fn go_back(&mut self) -> Result<()> {
        let starting_directory = self.WORKING_DIRECTORY.clone();
        if let Some(previous_path) = self.backward_directories.pop_back() {
            self.WORKING_DIRECTORY = previous_path;
            self.forward_directories.push_front(starting_directory);
            self.update_process_env_vars(false, false, true)
        } else {
            Err(ShellError::NoPreviousDirectory.into())
        }
    }

    // Sets the current working directory to the next working directory
    pub fn go_forward(&mut self) -> Result<()> {
        let starting_directory = self.WORKING_DIRECTORY.clone();
        if let Some(next_path) = self.forward_directories.pop_front() {
            self.WORKING_DIRECTORY = next_path;
            self.backward_directories.push_back(starting_directory);
            self.update_process_env_vars(false, false, true)
        } else {
            Err(ShellError::NoNextDirectory.into())
        }
    }
}

// Gets the name of the user who invoked the shell (to be used when the shell is first initialized)
fn get_parent_env_var(var_name: &str) -> Result<String> {
    std::env::var(var_name).map_err(|_| ShellError::MissingExternalEnvironmentVariables.into())
}

// Converts the PATH environment variable from a string to a vector of Paths
fn convert_path(path: &str, home: &PathBuf) -> Result<VecDeque<Path>> {
    path.split(':')
        .map(|p| -> Result<Path> { Path::from_str(p, home) })
        .collect()
}
