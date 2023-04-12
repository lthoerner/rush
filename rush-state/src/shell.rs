use anyhow::Result;

use crate::config::Configuration;
use crate::environment::Environment;

// Represents the shell, its state, and provides methods for interacting with it
// ? Should this be called ShellState or something like that?
// TODO: Miscellaneous shell state like command_success, command_history etc might be better off in some sort of bundle struct
pub struct Shell {
    environment: Environment,
    config: Configuration,
    command_success: bool,
    command_history: Vec<String>,
}

impl Shell {
    pub fn new() -> Result<Self> {
        let config = Configuration::from_file("config/config.rush").unwrap_or(Configuration::default());

        Ok(Self {
            environment: Environment::new()?,
            config,
            command_success: true,
            command_history: Vec::new(),
        })
    }
}

// Wrapper struct around all of the shell data that could be needed for any command to run
// For instance, a command like 'config' may need to access the shell's environment, whereas
// a command like 'exit' may not need any data at all, but the data needs to be available in all cases
pub struct Context<'a> {
    shell: &'a mut Shell,
}

#[allow(non_snake_case)]
impl<'a> Context<'a> {
    pub fn new(shell: &'a mut Shell) -> Self {
        Self { shell }
    }

    pub fn env(&self) -> &Environment {
        &self.shell.environment
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.shell.environment
    }

    pub fn shell_config(&self) -> &Configuration {
        &self.shell.config
    }

    pub fn shell_config_mut(&mut self) -> &mut Configuration {
        &mut self.shell.config
    }

    pub fn success(&self) -> bool {
        self.shell.command_success
    }

    pub fn set_success(&mut self, success: bool) {
        self.shell.command_success = success;
    }

    pub fn history(&self) -> &Vec<String> {
        &self.shell.command_history
    }

    pub fn history_add(&mut self, command: String) {
        self.shell.command_history.push(command);
    }
}
