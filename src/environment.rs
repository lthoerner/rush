use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;

use crate::errors::ShellError;
use crate::path::Path;

// Represents the shell environment by encapsulating the environment variables
#[allow(dead_code)]
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
        // ? Why was this using Path::new() instead of Path::from_str_path()?
        let working_directory = Path::from_str_path(get_parent_env_var("PWD")?.as_str(), &home)?;

        Ok(Self {
            user,
            home,
            working_directory,
            previous_working_directory: None,
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
            std::env::set_var("USER", &self.user);
        }

        if set_home {
            std::env::set_var("HOME", &self.home);
        }

        if set_working_directory {
            std::env::set_current_dir(self.working_directory.absolute())
                .map_err(|_| ShellError::FailedToUpdateEnvironmentVariables)?;
        }

        Ok(())
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
        self.update_process_env_vars(false, false, true)
    }
}

// Gets the name of the user who invoked the shell (to be used when the shell is first initialized)
fn get_parent_env_var(var_name: &str) -> Result<String> {
    std::env::var(var_name).map_err(|_| ShellError::MissingExternalEnvironmentVariables.into())
}
