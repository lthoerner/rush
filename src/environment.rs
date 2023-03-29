#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use crate::path::Path;

// Represents the shell environment by encapsulating the environment variables
pub struct Environment {
    pub user: String,
    home: PathBuf,
    pub working_directory: Path,
    custom_variables: HashMap<String, String>,
}

impl Default for Environment {
    fn default() -> Self {
        let user = get_caller_env_var("USER");
        let home = PathBuf::from(get_caller_env_var("HOME"));
        let working_directory = Path::new(PathBuf::from(get_caller_env_var("PWD")), &home);

        Self {
            user,
            home,
            working_directory,
            custom_variables: HashMap::new(),
        }
    }
}

impl Environment {
    // Updates the shell process's environment variables to match the internal representation
    pub fn update_process_env_vars(&self) {
        std::env::set_var("USER", &self.user);
        std::env::set_var("HOME", &self.home);
        std::env::set_current_dir(self.working_directory.absolute())
            .expect("Failed to set working directory");
    }

    pub fn home(&self) -> &PathBuf {
        &self.home
    }
}

// Gets the name of the current user
fn get_caller_env_var(var_name: &str) -> String {
    match std::env::var(var_name) {
        Ok(value) => value,
        Err(_) => {
            eprintln!(
                "[FATAL ERROR] Could not acquire required environment variable '{}'",
                var_name
            );
            std::process::exit(1);
        }
    }
}
