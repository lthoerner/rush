#![allow(dead_code)]

use std::collections::HashMap;

use crate::path::Path;

// Represents the shell environment by encapsulating the environment variables
pub struct Environment {
    user: String,
    working_directory: Path,
    custom_variables: HashMap<String, String>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            user: get_caller_user(),
            working_directory: Path::from_cwd(),
            custom_variables: HashMap::new(),
        }
    }
}

impl Environment {
    pub fn user(&self) -> &String {
        &self.user
    }

    pub fn user_mut(&mut self) -> &mut String {
        &mut self.user
    }

    pub fn working_directory(&self) -> &Path {
        &self.working_directory
    }

    pub fn working_directory_mut(&mut self) -> &mut Path {
        &mut self.working_directory
    }
}

// Gets the name of the current user
fn get_caller_user() -> String {
    std::env::var("USER").expect("Failed to get user")
}
