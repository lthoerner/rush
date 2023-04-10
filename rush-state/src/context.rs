use std::path::PathBuf;

use crate::config::Configuration;
use crate::environment::Environment;
use crate::path::Path;

// Wrapper struct around all of the shell data that could be needed for any command to run
// For instance, a command like 'config' may need to access the shell's environment, whereas
// a command like 'exit' may not need any data at all, but the data needs to be available in all cases
pub struct Context<'a> {
    environment: &'a mut Environment,
    config: &'a mut Configuration,
    pub command_success: &'a mut bool,
}

#[allow(non_snake_case)]
impl<'a> Context<'a> {
    pub fn new(
        environment: &'a mut Environment,
        config: &'a mut Configuration,
        command_success: &'a mut bool,
    ) -> Self {
        Self {
            environment,
            config,
            command_success,
        }
    }

    // Shortcut for accessing Context.shell.environment.HOME
    pub fn HOME(&self) -> &PathBuf {
        &self.environment.HOME()
    }

    // Shortcut for accessing Context.shell.environment
    pub fn env(&self) -> &Environment {
        &self.environment
    }

    // Mutable variant of Context.env()
    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.environment
    }

    // Shortcut for accessing Context.shell.config
    pub fn shell_config(&self) -> &Configuration {
        &self.config
    }

    // Mutable variant of Context.shell_config()
    pub fn shell_config_mut(&mut self) -> &mut Configuration {
        &mut self.config
    }

    // Shortcut for accessing Context.shell.environment.WORKING_DIRECTORY
    pub fn CWD(&self) -> &Path {
        &self.environment.WORKING_DIRECTORY
    }

    // Mutable variant of Context.CWD
    pub fn CWD_mut(&mut self) -> &mut Path {
        &mut self.environment.WORKING_DIRECTORY
    }

    // Setter for Context.command_success
    pub fn set_command_success(&mut self, success: bool) {
        *self.command_success = success;
    }
}
