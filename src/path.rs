use std::env::var;

// Wrapper class for a directory path string
pub struct Path {
    full_path: String,
    home_directory: String,
    shortened_path: String,
}

impl Path {
    pub fn from_cwd() -> Self {
        let full_path = get_env_cwd();
        let home_directory = get_env_home_directory();
        let shortened_path = collapse_home_directory(&full_path, &home_directory);

        Self {
            full_path,
            home_directory,
            shortened_path,
        }
    }

    pub fn full(&self) -> &String {
        &self.full_path
    }

    pub fn short(&self) -> &String {
        &self.shortened_path
    }
}

fn collapse_home_directory(full_path: &String, home_directory: &String) -> String {
    if full_path.starts_with(home_directory) {
        return full_path.replace(home_directory, "~")
    }

    full_path.clone()
}

fn get_env_cwd() -> String {
    var("PWD").expect("Failed to get path")
}

fn get_env_home_directory() -> String {
    var("HOME").expect("Failed to get home directory")
}