#![allow(dead_code)]

use std::fmt::{Display, Formatter};
use std::path::PathBuf;

// Wrapper class for a directory path string
#[derive(Hash, Eq, PartialEq)]
pub struct Path {
    absolute_path: PathBuf,
    // TODO: Figure out how this ties into Environment
    home_directory: String,
    shortened_path: String,
    truncation_factor: Option<usize>,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.absolute_path.display())
    }
}

impl Path {
    // Constructs a Path from the current working directory
    pub fn from_cwd() -> Self {
        let full_path = get_caller_cwd();
        let home_directory = get_caller_home_directory();

        let mut path = Self {
            absolute_path: full_path,
            home_directory,
            shortened_path: String::new(),
            truncation_factor: None,
        };

        path.update_shortened_path();

        path
    }

    // Gets the full path, with all directory names included
    pub fn full(&self) -> &PathBuf {
        &self.absolute_path
    }

    // Gets the shortened version of the path
    // If truncation is enabled, the path will be truncated
    // The shortened path will always have the home directory collapsed
    pub fn short(&self) -> &String {
        &self.shortened_path
    }

    // Sets the Path truncation factor
    pub fn set_truncation(&mut self, factor: usize) {
        self.truncation_factor = Some(factor);
        self.update_shortened_path();
    }

    // Disables Path truncation
    pub fn disable_truncation(&mut self) {
        self.truncation_factor = None;
        self.update_shortened_path();
    }

    // Re-generates the shortened path based on the current settings
    fn update_shortened_path(&mut self) {
        // ? Is there a less redundant way to write this?
        let path = match self.absolute_path.strip_prefix(&self.home_directory) {
            Ok(path) => {
                let mut path_string = path.to_string_lossy().to_string();
                // ? Is this really necessary? Wouldn't it be fine to just have '~/'?
                path_string = match path_string.len() {
                    0 => String::from("~"),
                    _ => format!("~/{}", path_string),
                };

                path_string
            }
            Err(_) => self.absolute_path.to_string_lossy().to_string(),
        };

        // ! This might cause a bug with directories that have a '/' in their name
        // ! Also might cause a bug with non-unicode characters (paths use OsString which is not guaranteed to be valid unicode)
        let directories: Vec<String> = path.split("/").map(|d| d.to_string()).collect();
        let mut truncated_directories = Vec::new();

        if let Some(factor) = self.truncation_factor {
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

        let truncated_directories = truncated_directories.join("/");

        self.shortened_path = truncated_directories
    }

    // Updates the Path using a new full path
    // TODO: Result instead of bool for error handling
    pub fn set_path(&mut self, new_full_path: &str) -> bool {
        // Home directory shorthand must be expanded before setting the path,
        // because PathBuf is not user-specific and only uses absolute paths
        let new_absolute_path = PathBuf::from(self.expand_home(new_full_path));

        if !new_absolute_path.exists() {
            return false;
        }

        self.absolute_path = new_absolute_path;
        self.update_shortened_path();

        true
    }

    fn expand_home(&self, path: &str) -> String {
        if path.starts_with("~") {
            return path.replace("~", &self.home_directory);
        }

        path.to_string()
    }
}

fn get_caller_cwd() -> PathBuf {
    PathBuf::from(std::env::var("PWD").expect("Failed to get path"))
}

fn get_caller_home_directory() -> String {
    std::env::var("HOME").expect("Failed to get home directory")
}
