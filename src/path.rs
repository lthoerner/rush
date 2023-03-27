#![allow(dead_code)]

use std::env::var;
use std::fmt::{Display, Formatter};

// Wrapper class for a directory path string
#[derive(Hash, Eq, PartialEq)]
pub struct Path {
    full_path: String,
    // TODO: Figure out how this ties into Environment
    home_directory: String,
    shortened_path: String,
    truncation_factor: Option<usize>,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.full_path)
    }
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
            truncation_factor: None,
        }
    }

    // Gets the full path, with all directory names included
    pub fn full(&self) -> &String {
        &self.full_path
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
        let path = collapse_home_directory(&self.full_path, &self.home_directory);
        // ! This might cause a bug with directories that have a '/' in their name
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
    pub fn set_path(&mut self, new_full_path: &str) {
        self.full_path = new_full_path.to_string();
        self.update_shortened_path();
    }
}

// ? Should this be turned into a method?
fn collapse_home_directory(full_path: &String, home_directory: &String) -> String {
    if full_path.starts_with(home_directory) {
        return full_path.replace(home_directory, "~");
    }

    full_path.clone()
}

fn get_env_cwd() -> String {
    var("PWD").expect("Failed to get path")
}

fn get_env_home_directory() -> String {
    var("HOME").expect("Failed to get home directory")
}
