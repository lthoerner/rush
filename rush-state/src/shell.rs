use anyhow::Result;

use crate::config::Configuration;
use crate::environment::Environment;

// Represents the shell, its state, and provides methods for interacting with it
// ? Should this be called ShellState or something like that?
pub struct Shell {
    pub environment: Environment,
    pub config: Configuration,
    pub success: bool,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Ok(Self {
            environment: Environment::new()?,
            config: Configuration::from_file("config/config.rush")?,
            success: true,
        })
    }
}
