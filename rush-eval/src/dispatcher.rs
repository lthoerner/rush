use anyhow::Result;

use rush_state::context::Context;
use rush_state::path::Path;
use rush_state::errors::ShellError;

use crate::commands::{Runnable, Builtin};
use crate::builtins;

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

    // Evaluates and executes a command from a string
    pub fn eval(&self, context: &mut Context, command_name: String, command_args: Vec<String>) -> Result<()> {
        let command_name = command_name.as_str();
        let command_args = command_args.iter().map(|a| a.as_str()).collect();

        // Dispatch the command to the Dispatcher
        self.dispatch(command_name, command_args, context)
    }

    // Resolves and dispatches a command to the appropriate function or external binary
    // If the command does not exist, returns None
    fn dispatch(
        &self,
        command_name: &str,
        command_args: Vec<&str>,
        context: &mut Context,
    ) -> Result<()> {
        // If the command resides in the Dispatcher (generally means it is a builtin) run it
        if let Some(command) = self.resolve(command_name) {
            let exit_status = command.run(context, command_args);
            exit_status
        } else {
            // If the command is not in the Dispatcher, try to run it as an executable from the PATH
            let path = Path::from_path_var(command_name, context.env().PATH());
            if let Ok(path) = path {
                // ? Should this check if the file is an executable first?
                Runnable::External(path).run(context, command_args)
            } else {
                Err(ShellError::UnknownCommand(command_name.to_string()).into())
            }
        }
    }
}
