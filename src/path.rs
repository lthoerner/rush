use std::env::var;

// Wrapper class for a directory path string
pub struct Path {
    full_path: String,
    home_directory: String,
}

impl Path {
    pub fn from_cwd() -> Self {
        Self {
            full_path: get_env_cwd(),
            home_directory: get_env_home_directory(),
        }
    }

    pub fn full(&self) -> String {
        self.full_path.clone()
    }

    pub fn short(&self) -> String {
        if self.full_path.starts_with(&self.home_directory) {
            self.full_path.replace(&self.home_directory, "~")
        } else {
            self.full_path.clone()
        }
    }
}

fn get_env_cwd() -> String {
    var("PWD").expect("Failed to get path")
}

fn get_env_home_directory() -> String {
    var("HOME").expect("Failed to get home directory")
}