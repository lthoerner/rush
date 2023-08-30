use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

use fs_err::File;

use crate::errors::{Handle, Result};

// Represents any settings for the shell, most of which can be configured by the user
pub struct Configuration {
    // The truncation length for the prompt
    pub truncation_factor: Option<usize>,
    // Whether to show the prompt tick on a new line
    pub multi_line_prompt: bool,
    // How many directories to store in the back/forward history
    pub history_limit: Option<usize>,
    // Whether or not to print out full error messages and status codes when a command fails
    pub show_errors: bool,
    /// Paths to recursively search for plugins
    pub plugin_paths: Vec<PathBuf>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            truncation_factor: None,
            multi_line_prompt: false,
            history_limit: None,
            show_errors: true,
            plugin_paths: vec![],
        }
    }
}

impl Configuration {
    // Scans a configuration file for settings and updates the configuration accordingly
    pub fn from_file(filename: &str) -> Result<Self> {
        let filename = PathBuf::from(filename);
        let dirname = filename.parent().unwrap();

        let mut config = Self::default();
        let file = File::open(filename.clone())
            .replace_err_no_context(state_err!(FailedToOpenConfigFile(filename.clone())))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line =
                line.replace_err_no_context(state_err!(FailedToOpenConfigFile(filename.clone())))?;
            let tokens = line.split(": ").collect::<Vec<&str>>();
            if tokens.len() != 2 {
                return Err(state_err!(FailedToReadConfigFile(filename)));
            }

            let (key, value) = (tokens[0], tokens[1]);

            // ? Should these be underscores instead of hyphens?
            // TODO: Error handling
            match key {
                "truncation-factor" => {
                    if let Ok(length) = value.parse::<usize>() {
                        config.truncation_factor = Some(length);
                    } else if value == "false" {
                        config.truncation_factor = None;
                    }
                }
                "multi-line-prompt" => {
                    if let Ok(multi_line) = value.parse::<bool>() {
                        config.multi_line_prompt = multi_line;
                    }
                }
                "history-limit" => {
                    if let Ok(limit) = value.parse::<usize>() {
                        config.history_limit = Some(limit);
                    } else if value == "false" {
                        config.history_limit = None;
                    }
                }
                "show-errors" => {
                    if let Ok(show) = value.parse::<bool>() {
                        config.show_errors = show;
                    }
                }
                "plugin-path" => {
                    config.plugin_paths.push(dirname.join(value));
                }
                _ => return Err(state_err!(FailedToReadConfigFile(filename))),
            }
        }

        Ok(config)
    }
}
