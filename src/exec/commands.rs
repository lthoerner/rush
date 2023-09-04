use std::process::Command as Process;

use crate::errors::{Handle, Result};
use crate::state::{Path, ShellState};

/// Represents either a builtin (internal command) or an executable (external command)
/// A `Runnable` may be executed by calling its `.run()` method
pub trait Runnable {
    fn run(&self, shell: &mut ShellState, arguments: Vec<&str>) -> Result<()>;
}

/// Wrapper type that makes it easier to read code related to builtins
pub struct Aliases {
    aliases: Vec<String>,
}

// * This implementation is here to make it easier to define aliases using string literals
impl From<Vec<&str>> for Aliases {
    fn from(aliases: Vec<&str>) -> Self {
        Self {
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
        }
    }
}

impl Aliases {
    pub fn contains(&self, alias: &str) -> bool {
        self.aliases.contains(&alias.to_string())
    }
}

/// Represents a builtin function, its name and its aliases
pub struct Builtin {
    pub true_name: String,
    pub aliases: Aliases,
    #[allow(clippy::type_complexity)]
    function: Box<dyn Fn(&mut ShellState, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    pub fn new<F: Fn(&mut ShellState, Vec<&str>) -> Result<()> + 'static>(
        true_name: &str,
        aliases: Vec<&str>,
        function: F,
    ) -> Self {
        let true_name = true_name.to_string();
        let aliases = Aliases::from(aliases);
        let function = Box::new(function);

        Self {
            true_name,
            aliases,
            function,
        }
    }
}

impl Runnable for Builtin {
    fn run(&self, shell: &mut ShellState, arguments: Vec<&str>) -> Result<()> {
        (self.function)(shell, arguments)
    }
}

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
        let mut process = Process::new(self.path.path())
            .args(arguments)
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
