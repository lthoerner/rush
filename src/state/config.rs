use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
};

use fs_err::File;

use crate::errors::{Handle, Result};

/// Represents any settings for the shell, most of which can be configured by the user
pub struct Configuration {
    /// The truncation length for the prompt
    pub truncation: Option<usize>,
    /// How many directories to store in the back/forward history
    pub history_limit: Option<usize>,
    /// Whether to show the prompt tick on a new line
    pub multiline_prompt: bool,
    /// Whether or not to print out full error messages and status codes when a command fails
    pub show_errors: bool,
    /// Paths to recursively search for plugins
    pub plugin_paths: Vec<PathBuf>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            truncation: None,
            history_limit: None,
            multiline_prompt: false,
            show_errors: true,
            plugin_paths: vec![],
        }
    }
}

impl Configuration {
    /// Scans a configuration file for settings and updates the configuration accordingly
    pub fn from_file(filename: &str) -> Result<Self> {
        let filename = PathBuf::from(filename);
        let open_error_msg = format!("Config file '{}' could not be opened", filename.display());
        let read_error_msg = format!("Config file '{}' could not be read", filename.display());

        let dirname = filename
            .parent()
            .replace_err(|| file_err!(CouldNotGetParent: filename))?;

        let mut config = Self::default();
        let file = File::open(&filename)
            .replace_err_with_msg(|| file_err!(CouldNotOpenFile: filename), &open_error_msg)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line
                .replace_err_with_msg(|| file_err!(CouldNotReadFile: filename), &read_error_msg)?;
            let tokens = line.split(": ").collect::<Vec<&str>>();
            if tokens.len() != 2 {
                return Err(file_err!(CouldNotReadFile: filename).set_context(&read_error_msg));
            }

            let (key, value) = (tokens[0], tokens[1]);

            // ? Should these be underscores instead of hyphens?
            match key {
                "truncation" => {
                    if let Ok(length) = value.parse::<usize>() {
                        config.truncation = Some(length);
                    } else if value == "false" {
                        config.truncation = None;
                    } else {
                        return Err(
                            file_err!(CouldNotReadFile: filename).set_context(&read_error_msg)
                        );
                    }
                }
                "history-limit" => {
                    if let Ok(limit) = value.parse::<usize>() {
                        config.history_limit = Some(limit);
                    } else if value == "false" {
                        config.history_limit = None;
                    }
                }
                "multiline-prompt" => {
                    config.multiline_prompt = value.parse::<bool>().replace_err_with_msg(
                        || file_err!(CouldNotReadFile: filename),
                        &read_error_msg,
                    )?;
                }
                "show-errors" => {
                    config.show_errors = value.parse::<bool>().replace_err_with_msg(
                        || file_err!(CouldNotReadFile: filename),
                        &read_error_msg,
                    )?;
                }
                "plugin-path" => {
                    config.plugin_paths.push(dirname.join(value));
                }
                _ => return Err(file_err!(CouldNotReadFile: filename).set_context(&read_error_msg)),
            }
        }

        Ok(config)
    }
}
