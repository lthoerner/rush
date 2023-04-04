use std::path::PathBuf;
use std::process::Command as StdCommand;

use anyhow::Result;

use crate::builtins;
use crate::environment::Environment;
use crate::errors::ExternalCommandError;
use crate::path::{self, Path};
use crate::shell::Shell;

// Represents a command that can be run by the prompt
pub struct Command {
    true_name: String,
    aliases: Vec<String>,
    runnable: Runnable,
}

impl Command {
    fn new(true_name: &str, aliases: Vec<&str>, runnable: Runnable) -> Self {
        let true_name = true_name.to_string();
        let aliases = aliases.iter().map(|a| a.to_string()).collect();

        Self {
            true_name,
            aliases,
            runnable,
        }
    }

    #[allow(dead_code)]
    pub fn true_name(&self) -> &String {
        &self.true_name
    }
}

// Represents either an internal command or an external binary that can be invoked by a command
enum Runnable {
    Internal(Box<dyn Fn(&mut Context, Vec<&str>) -> Result<()>>),
    External(PathBuf),
}

impl Runnable {
    // Constructs an Internal Runnable from a function
    fn internal<F: Fn(&mut Context, Vec<&str>) -> Result<()> + 'static>(function: F) -> Self {
        Self::Internal(Box::new(function))
    }

    // Constructs an External Runnable from a path
    fn external(path: PathBuf) -> Self {
        Self::External(path)
    }

    // Executes either a builtin or a binary
    fn run(&self, context: &mut Context, arguments: Vec<&str>) -> Result<()> {
        match self {
            Runnable::Internal(command_function) => command_function(context, arguments),
            Runnable::External(path) => {
                let output = StdCommand::new(path).args(arguments).output();
                match output {
                    Err(err) => {
                        if let Some(err_code) = err.raw_os_error() {
                            Err(ExternalCommandError::FailedToExecute(err_code as isize).into())
                        } else {
                            // ? What does this error mean? What should the error code be? Should it use a different variant like UnknownError?
                            Err(ExternalCommandError::FailedToExecute(-1).into())
                        }
                    }
                    Ok(output) => {
                        let stdout = output.stdout;
                        let string = String::from_utf8_lossy(stdout.as_slice());
                        eprint!("{}", string);
                        Ok(())
                    }
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
    pub fn env(&self) -> &Environment {
        &self.shell.environment
    }

    // Mutable variant of Context.env()
    pub fn env_mut(&mut self) -> &mut Environment {
        &mut self.shell.environment
    }

    // Shortcut for accessing Context.shell.environment.working_directory
    pub fn cwd(&self) -> &Path {
        &self.shell.environment.working_directory
    }

    // Mutable variant of Context.cwd()
    pub fn cwd_mut(&mut self) -> &mut Path {
        &mut self.shell.environment.working_directory
    }
}

// Represents a collection of commands
// Allows for command resolution through aliases
pub struct CommandManager {
    commands: Vec<Command>,
}

impl Default for CommandManager {
    // Initializes the command manager with the default shell commands and aliases
    fn default() -> Self {
        let mut manager = Self::new();

        manager.add_command("test", vec!["t"], Runnable::internal(builtins::test));
        manager.add_command(
            "exit",
            vec!["quit", "q"],
            Runnable::internal(builtins::exit),
        );
        manager.add_command(
            "working-directory",
            vec!["pwd", "wd"],
            Runnable::internal(builtins::working_directory),
        );
        manager.add_command(
            "change-directory",
            vec!["cd"],
            Runnable::internal(builtins::change_directory),
        );
        manager.add_command(
            "list-directory",
            vec!["directory", "list", "ls", "dir"],
            Runnable::internal(builtins::list_directory),
        );
        manager.add_command(
            "go-back",
            vec!["back", "b", "prev", "pd"],
            Runnable::internal(builtins::go_back),
        );
        manager.add_command(
            "clear-terminal",
            vec!["clear", "cls"],
            Runnable::internal(builtins::clear_terminal),
        );
        manager.add_command(
            "create-file",
            vec!["create", "touch", "new", "cf"],
            Runnable::internal(builtins::create_file),
        );
        manager.add_command(
            "create-directory",
            // TODO: Figure out 'cd' alias conflict
            vec!["mkdir", "md"],
            Runnable::internal(builtins::create_directory),
        );
        manager.add_command(
            "delete-file",
            vec!["delete", "remove", "rm", "del", "df"],
            Runnable::internal(builtins::delete_file),
        );
        manager.add_command(
            "read-file",
            vec!["read", "cat", "rf"],
            Runnable::internal(builtins::read_file),
        );
        manager.add_command(
            "truncate",
            vec!["trunc"],
            Runnable::internal(builtins::truncate),
        );
        manager.add_command(
            "untruncate",
            vec!["untrunc"],
            Runnable::internal(builtins::untruncate),
        );

        manager
    }
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    // Adds a command to the manager
    fn add_command(&mut self, true_name: &str, aliases: Vec<&str>, runnable: Runnable) {
        self.commands
            .push(Command::new(true_name, aliases, runnable));
    }

    // Resolves a command name to a command
    // Returns None if the command is not found
    fn resolve(&self, command_name: &str) -> Option<&Command> {
        for command in &self.commands {
            if command.true_name == command_name {
                return Some(command);
            }

            for alias in &command.aliases {
                if alias == command_name {
                    return Some(command);
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
        if let Some(command) = self.resolve(command_name) {
            return Some(command.runnable.run(context, command_args));
        } else {
            // ? Should the path be canonicalized in the Runnable::External constructor or here?
            let path = path::resolve(command_name, context.home());
            if let Some(path) = path {
                // ? Should we check if the file is an executable first?
                let runnable = Runnable::external(path);
                Some(runnable.run(context, command_args))
            } else {
                None
            }
        }
    }
}
