use std::sync::{Arc, RwLock};

use crossterm::style::Stylize;

use super::config::Configuration;
use super::environment::Environment;
use super::Path;
use crate::errors::Result;

/// Represents the shell state and provides methods for interacting with it
pub struct ShellState {
    pub environment: Environment,
    pub config: Configuration,
    pub last_command_succeeded: bool,
    pub should_exit: bool,
}

impl ShellState {
    pub fn new() -> Result<Arc<RwLock<Self>>> {
        let config =
            Configuration::from_file("./config/config.rush").unwrap_or(Configuration::default());

        Ok(Arc::new(RwLock::new(Self {
            environment: Environment::new()?,
            config,
            last_command_succeeded: true,
            should_exit: false,
        })))
    }

    /// Generates the prompt string used by the `LineEditor`
    pub fn generate_prompt(&self) -> String {
        let user = self.environment.USER.clone();
        let home = &self.environment.HOME;
        let truncation = self.config.truncation;
        let cwd = self.CWD().collapse(home, truncation);
        let prompt_delimiter = match self.config.multiline_prompt {
            true => "\n",
            false => " ",
        };

        // ? What is the actual name for this?
        let prompt_tick = match self.last_command_succeeded {
            true => "❯".green(),
            false => "❯".red(),
        }
        .bold();

        format!(
            "\n{} on {}{}{} ",
            user.dark_blue(),
            cwd.dark_green(),
            prompt_delimiter,
            prompt_tick
        )
    }

    /// Convenience getter for the current working directory
    #[allow(non_snake_case)]
    pub fn CWD(&self) -> &Path {
        self.environment.CWD()
    }
}
