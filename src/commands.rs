use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;

use crate::builtins;
use crate::environment::Environment;
use crate::errors::ExternalCommandError;
use crate::path::Path;
use crate::shell::Shell;

// Represents a builtin function, its name and its aliases
pub struct Builtin {
    true_name: String,
    aliases: Vec<String>,
    function: Box<dyn Fn(&mut Context, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    fn new<F: Fn(&mut Context, Vec<&str>) -> Result<()> + 'static>(
        true_name: &str,
        aliases: Vec<&str>,
        function: F,
    ) -> Self {
        let true_name = true_name.to_string();
        let aliases = aliases.iter().map(|a| a.to_string()).collect();
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
                // Create the process and pass the provided arguments to it
                let mut executable = Command::new(path.path());
                executable.args(arguments);
                // Execute the process and wait for it to finish
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

// Wrapper struct around all of the data that could be needed for any command to run
// For instance, a command like 'truncate' may need to access the working directory, whereas
// a command like 'exit' may not need any data at all, but the data needs to be available in all cases
// TODO: Add an example for a command that needs different information
pub struct Context<'a> {
    pub shell: &'a mut Shell,
}

impl<'a> Context<'a> {
    pub fn new(shell: &'a mut Shell) -> Self {
        Self { shell }
    }

    // Shortcut for accessing Context.shell.environment.home
    pub fn home(&self) -> &PathBuf {
        &self.shell.environment.home()
    }

    // Shortcut for accessing Context.shell.environment
    #[allow(dead_code)]
    pub fn env(&self) -> &Environment {
        &self.shell.environment
    }

    // Mutable variant of Context.env()
    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.shell.environment
    }

    // Shortcut for accessing Context.shell.environment.working_directory
    pub fn cwd(&self) -> &Path {
        &self.shell.environment.WORKING_DIRECTORY
    }

    // Mutable variant of Context.cwd()
    #[allow(dead_code)]
    pub fn cwd_mut(&mut self) -> &mut Path {
        &mut self.shell.environment.WORKING_DIRECTORY
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
        dispatcher.add_builtin("truncate", vec!["trunc"], builtins::truncate);
        dispatcher.add_builtin("untruncate", vec!["untrunc"], builtins::untruncate);

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

                for alias in &builtin.aliases {
                    if alias == command_name {
                        return Some(command);
                    }
                }
            }
        }

        None
    }

    // Resolves and dispatches a command to the appropriate function or external binary
    // If the command does not exist, returns None
    // ? How should I consume the Context to ensure that it is not used after the command is run?
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
            let path = Path::from_path_var(command_name, context.env().path());
            if let Ok(path) = path {
                // ? Should this check if the file is an executable first?
                Some(Runnable::External(path).run(context, command_args))
            } else {
                None
            }
        }
    }
}
