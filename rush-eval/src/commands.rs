use std::process::Command as Process;

use anyhow::Result;

use rush_state::context::Context;
use rush_state::errors::ExternalCommandError;
use rush_state::path::Path;

// Wrapper type for Vec<String> that makes it easier to read code related to Builtins
pub struct Aliases {
    aliases: Vec<String>,
}

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
    function: Box<dyn Fn(&mut Context, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    pub fn new<F: Fn(&mut Context, Vec<&str>) -> Result<()> + 'static>(
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

// Represents either a builtin (internal command) or an executable (external command)
// A Runnable may be executed by calling its .run() method
pub enum Runnable {
    // * This variant is only used for shell builtins, as defined in Dispatcher::default()
    Internal(Builtin),
    // * This variant is used in two cases:
    // * 1. When the user invokes an external binary using the run-executable builtin (explicit invocation)
    // * 2. When the user invokes an external binary that is in the PATH without using the run-executable builtin (implicit invocation)
    // * A Path (from the path module) must be validated before construction, so it is safe to simply place it in an External
    External(Path),
}

impl Runnable {
    // Executes either a builtin or a binary
    pub fn run(&self, context: &mut Context, arguments: Vec<&str>) -> Result<()> {
        match self {
            Runnable::Internal(builtin) => (builtin.function)(context, arguments),
            Runnable::External(path) => {
                // Create the Process and pass the provided arguments to it
                let mut executable = Process::new(path.path());
                executable.args(arguments);
                // Execute the Process and wait for it to finish
                let mut handle = executable.spawn()?;
                let status = handle.wait()?;

                if status.success() {
                    Ok(())
                } else {
                    // * 126 is a special exit code that means that the command was found but could not be executed
                    // * as per https://tldp.org/LDP/abs/html/exitcodes.html
                    // * It can be assumed that the command was found here because the External path must have been validated already
                    // * Otherwise it could be a 127 for "command not found"
                    Err(ExternalCommandError::FailedToExecute(status.code().unwrap_or(126) as isize).into())
                }
            }
        }
    }
}
