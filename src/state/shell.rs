use anyhow::Result;
use crossterm::style::Stylize;

use super::config::Configuration;
use super::environment::Environment;

// Represents the shell state and provides methods for interacting with it
// ? Miscellaneous shell state like command_success, command_history etc might be better off in some sort of bundle struct
pub struct ShellState {
    pub environment: Environment,
    pub config: Configuration,
    pub last_command_succeeded: bool,
    pub should_exit: bool,
}

impl ShellState {
    pub fn new() -> Result<Self> {
        let config =
            Configuration::from_file("./config/config.rush").unwrap_or(Configuration::default());

        Ok(Self {
            environment: Environment::new()?,
            config,
            last_command_succeeded: true,
            should_exit: false,
        })
    }

    // Generates the prompt string used by the REPL
    pub fn generate_prompt(&self) -> String {
        let user = self.environment.USER.clone();
        let home = &self.environment.HOME;
        let truncation = self.config.truncation_factor;
        let cwd = self.environment.CWD.collapse(home, truncation);
        let prompt_delimiter = match self.config.multi_line_prompt {
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
}
