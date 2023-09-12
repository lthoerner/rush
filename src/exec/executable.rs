use std::process::Command as Process;

use super::Runnable;
use crate::errors::{Handle, Result};
use crate::state::{Path, ShellState};

/// Represents an executable (external command)
pub struct Executable {
    path: Path,
}

impl Executable {
    // * This constructor is used in two cases:
    // * 1. When the user invokes an external binary using the run-executable builtin (explicit invocation)
    // * 2. When the user invokes an external binary that is in the PATH without using the run-executable builtin (implicit invocation)
    // * The Path wrapper type must be validated before construction, so it can be assumed that the path is valid
    pub fn new(path: Path) -> Self {
        Self { path }
    }
}

impl Runnable for Executable {
    // * Executables do not have access to the shell state, but the context argument is required by the Runnable trait
    fn run(&self, _shell: &mut ShellState, arguments: Vec<&str>) -> Result<()> {
        // Create the Process, pass the provided arguments to it, and execute it
        // * Executable name has to be removed because `std::process::Command`
        // * automatically adds the executable name as the first argument
        let mut process = Process::new(self.path.path())
            .args(&arguments[1..])
            .spawn()
            .replace_err(|| executable_err!(PathNoLongerExists: self.path))?;

        let status = process
            .wait()
            .replace_err(|| executable_err!(CouldNotWait))?;

        match status.success() {
            true => Ok(()),
            false => {
                // * 126 is a special exit code that means that the command was found but could not be executed
                // * as per https://tldp.org/LDP/abs/html/exitcodes.html
                // * It can be assumed that the command was found here because the Executable path must have been validated already
                // * Otherwise it could be a 127 for "command not found"
                Err(executable_err!(FailedToExecute:
                    status.code().unwrap_or(126) as isize
                ))
            }
        }
    }
}
