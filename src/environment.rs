#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::errors::ShellError;
use crate::path::Path;

// Represents the shell environment by encapsulating the environment variables
pub struct Environment {
    user: String,
    home: PathBuf,
    pub working_directory: Path,
    // ? Should this just be a single path or should it store a history?
    pub previous_working_directory: Option<PathBuf>,
    custom_variables: HashMap<String, String>,
}

impl Environment {
    pub fn new() -> Result<Self> {
        let user = get_parent_env_var("USER")?;
        let home = PathBuf::from(get_parent_env_var("HOME")?);
        let working_directory = Path::new(PathBuf::from(get_parent_env_var("PWD")?), &home);

        Ok(Self {
            user,
            home,
            working_directory,
            previous_working_directory: None,
            custom_variables: HashMap::new(),
        })
    }

    // Updates the shell process's environment variables to match the internal representation
    // ? Should this have options of which variables to update?
    pub fn update_process_env_vars(&self) -> Result<()> {
        std::env::set_var("USER", &self.user);
        std::env::set_var("HOME", &self.home);
        std::env::set_current_dir(self.working_directory.absolute())
            .map_err(|_| ShellError::FailedToUpdateEnvironmentVariables.into())
    }

    pub fn user(&self) -> &String {
        &self.user
    }

    pub fn home(&self) -> &PathBuf {
        &self.home
    }

    // Sets the current working directory and stores the previous working directory
    pub fn set_path(&mut self, new_path: &str) -> Result<()> {
        let previous_path = self.working_directory.absolute().clone();
        self.working_directory.set_path(new_path)?;
        self.previous_working_directory = Some(previous_path);

        Ok(())
    }
}

// Gets the name of the user who invoked the shell (to be used when the shell is first initialized)
fn get_parent_env_var(var_name: &str) -> Result<String> {
    std::env::var(var_name).map_err(|_| ShellError::MissingExternalEnvironmentVariables.into())
}
