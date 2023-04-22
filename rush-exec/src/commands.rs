use std::process::Command as Process;

use anyhow::Result;

use rush_state::console::Console;
use rush_state::path::Path;
use rush_state::shell::Shell;

use crate::errors::ExecutableError;

// Represents either a builtin (internal command) or an executable (external command)
// A Runnable may be executed by calling its .run() method
pub trait Runnable {
    fn run(&self, shell: &mut Shell, console: &mut Console, arguments: Vec<&str>) -> Result<()>;
}

// Wrapper type for Vec<String> that makes it easier to read code related to Builtins
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

// Represents a builtin function, its name and its aliases
pub struct Builtin {
    pub true_name: String,
    pub aliases: Aliases,
    function: Box<dyn Fn(&mut Shell, &mut Console, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    pub fn new<F: Fn(&mut Shell, &mut Console, Vec<&str>) -> Result<()> + 'static>(
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
    fn run(&self, shell: &mut Shell, console: &mut Console, arguments: Vec<&str>) -> Result<()> {
        (self.function)(shell, console, arguments)
    }
}

// Represents an external binary/executable
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
    fn run(&self, _shell: &mut Shell, console: &mut Console, arguments: Vec<&str>) -> Result<()> {
        // Create the Process and pass the provided arguments to it
        let mut executable = Process::new(self.path.path());
        executable.args(arguments);
        // Execute the Process and wait for it to finish
        // TODO: There may be other types of errors that could happen, they may need handlers
        let Ok(handle) = executable.output() else { return Err(ExecutableError::PathNoLongerExists(self.path.path().clone()).into()); };

        if handle.status.success() {
            console.println(std::str::from_utf8(&handle.stdout)?);
            Ok(())
        } else {
            // * 126 is a special exit code that means that the command was found but could not be executed
            // * as per https://tldp.org/LDP/abs/html/exitcodes.html
            // * It can be assumed that the command was found here because the External path must have been validated already
            // * Otherwise it could be a 127 for "command not found"
            Err(
                ExecutableError::FailedToExecute(handle.status.code().unwrap_or(126) as isize)
                    .into(),
            )
        }
    }
}
