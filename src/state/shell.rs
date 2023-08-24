use anyhow::Result;

use super::config::Configuration;
use super::environment::Environment;

// Represents the shell state and provides methods for interacting with it
// TODO: Miscellaneous shell state like command_success, command_history etc might be better off in some sort of bundle struct
pub struct ShellState {
    pub(crate) environment: Environment,
    pub(crate) config: Configuration,
    pub(crate) command_success: bool,
}

impl ShellState {
    pub fn new() -> Result<Self> {
        let config =
            Configuration::from_file("./config/config.rush").unwrap_or(Configuration::default());

        Ok(Self {
            environment: Environment::new()?,
            config,
            command_success: true,
        })
    }

    pub fn env(&self) -> &Environment {
        &self.environment
    }

    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Configuration {
        &mut self.config
    }

    pub fn success(&self) -> bool {
        self.command_success
    }

    pub fn set_success(&mut self, success: bool) {
        self.command_success = success;
    }
}
