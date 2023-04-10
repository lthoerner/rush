use std::path::PathBuf;
use std::process::Command as Process;

use anyhow::Result;

use crate::{builtins, environment};
use crate::environment::Environment;
use crate::errors::ExternalCommandError;
use crate::path::Path;
use crate::shell::{Configuration, Shell};

// Wrapper type for Vec<String> that makes it easier to read code related to Builtins
struct Aliases {
    aliases: Vec<String>,
}

impl From<Vec<&str>> for Aliases {
    fn from(aliases: Vec<&str>) -> Self {
        Self { aliases: aliases.iter().map(|a| a.to_string()).collect() }
    }
}

impl Aliases {
    fn contains(&self, alias: &str) -> bool {
        self.aliases.contains(&alias.to_string())
    }
}

// Represents a builtin function, its name and its aliases
pub struct Builtin {
    true_name: String,
    aliases: Aliases,
    function: Box<dyn Fn(&mut Context, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    fn new<F: Fn(&mut Context, Vec<&str>) -> Result<()> + 'static>(
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

    #[allow(dead_code)]
    pub fn true_name(&self) -> &String {
        &self.true_name
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

// Wrapper struct around all of the shell data that could be needed for any command to run
// For instance, a command like 'config' may need to access the shell's environment, whereas
// a command like 'exit' may not need any data at all, but the data needs to be available in all cases
pub struct Context<'a> {
    environment: &'a mut Environment,
    config: &'a mut Configuration,
}

#[allow(non_snake_case)]
impl<'a> Context<'a> {
    pub fn new(environment: &'a mut Environment, config: &'a mut Configuration) -> Self {
        Self {
            environment,
            config,
        }
    }

    // Shortcut for accessing Context.shell.environment.HOME
    pub fn HOME(&self) -> &PathBuf {
        &self.environment.HOME()
    }

    // Shortcut for accessing Context.shell.environment
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn CWD_mut(&mut self) -> &mut Path {
        &mut self.environment.WORKING_DIRECTORY
    }
}

// Represents a collection of commands
// Allows for command resolution and execution through aliases
// * The Dispatcher generally only stores builtins, but it is capable of storing external Runnables, also known as executables or binaries
// * However, because they do not have any aliases, they would not be able to be resolved
pub struct Dispatcher {
    commands: Vec<Runnable>,
}

impl Default for Dispatcher {
    // Initializes the Dispatcher with the default shell commands and aliases
    #[rustfmt::skip]
    fn default() -> Self {
        let mut dispatcher = Self::new();

        dispatcher.add_builtin("test", vec!["t"], builtins::test);
        dispatcher.add_builtin("exit", vec!["quit", "q"], builtins::exit);
        dispatcher.add_builtin("working-directory", vec!["pwd", "wd"], builtins::working_directory);
        dispatcher.add_builtin("change-directory", vec!["cd"], builtins::change_directory);
        dispatcher.add_builtin("list-directory", vec!["directory", "list", "ls", "dir"], builtins::list_directory);
        dispatcher.add_builtin("go-back", vec!["back", "b", "prev", "pd"], builtins::go_back);
        dispatcher.add_builtin("go-forward", vec!["forward", "f", "next", "nd"], builtins::go_forward);
        dispatcher.add_builtin("clear-terminal", vec!["clear", "cls"], builtins::clear_terminal);
        dispatcher.add_builtin("create-file", vec!["create", "touch", "new", "cf"], builtins::create_file);
        // TODO: Figure out 'cd' alias conflict
        dispatcher.add_builtin("create-directory", vec!["mkdir", "md"], builtins::create_directory);
        dispatcher.add_builtin("delete-file", vec!["delete", "remove", "rm", "del", "df"], builtins::delete_file);
        dispatcher.add_builtin("read-file", vec!["read", "cat", "rf"], builtins::read_file);
        dispatcher.add_builtin("run-executable", vec!["run", "exec", "re"], builtins::run_executable);
        dispatcher.add_builtin("configure", vec!["config", "conf"], builtins::configure);
        dispatcher.add_builtin("environment-variable", vec!["environment", "env", "ev"], builtins::environment_variable);
        dispatcher.add_builtin("edit-path", vec!["path", "ep"], builtins::edit_path);

        dispatcher
    }
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    // Adds a builtin to the Dispatcher
    fn add_builtin<F: Fn(&mut Context, Vec<&str>) -> Result<()> + 'static>(
        &mut self,
        true_name: &str,
        aliases: Vec<&str>,
        function: F,
    ) {
        self.commands.push(Runnable::Internal(Builtin::new(
            true_name, aliases, function,
        )))
    }

    // Resolves a command name to a command
    // Returns None if the command is not found
    // TODO: Figure out nomenclature on commands vs runnables
    fn resolve(&self, command_name: &str) -> Option<&Runnable> {
        for command in &self.commands {
            if let Runnable::Internal(builtin) = command {
                if builtin.true_name == command_name {
                    return Some(command);
                }

                if builtin.aliases.contains(command_name) {
                    return Some(command);
                }
            }
        }

        None
    }

    // Resolves and dispatches a command to the appropriate function or external binary
    // If the command does not exist, returns None
    pub fn dispatch(
        &self,
        command_name: &str,
        command_args: Vec<&str>,
        context: &mut Context,
    ) -> Option<Result<()>> {
        // If the command resides in the Dispatcher (generally means it is a builtin) run it
        if let Some(command) = self.resolve(command_name) {
            let exit_status = command.run(context, command_args);
            return Some(exit_status);
        } else {
            // If the command is not in the Dispatcher, try to run it as an executable from the PATH
            let path = Path::from_path_var(command_name, context.env().PATH());
            if let Ok(path) = path {
                // ? Should this check if the file is an executable first?
                Some(Runnable::External(path).run(context, command_args))
            } else {
                None
            }
        }
    }
}