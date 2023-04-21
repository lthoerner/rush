use anyhow::Result;

use rush_exec::builtins;
use rush_exec::commands::{Builtin, Executable, Runnable};
use rush_state::path::Path;
use rush_state::shell::Context;

use crate::errors::DispatchError;
use crate::parser;

// Represents a collection of builtin commands
// Allows for command resolution and execution through aliases
pub struct Dispatcher {
    commands: Vec<Builtin>,
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
        dispatcher.add_builtin("previous-directory", vec!["back", "b", "prev", "pd"], builtins::go_back);
        dispatcher.add_builtin("next-directory", vec!["forward", "f", "next", "nd"], builtins::go_forward);
        dispatcher.add_builtin("clear-terminal", vec!["clear", "cls"], builtins::clear_terminal);
        dispatcher.add_builtin("make-file", vec!["create", "touch", "new", "mf"], builtins::make_file);
        dispatcher.add_builtin("make-directory", vec!["mkdir", "md"], builtins::make_directory);
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
        self.commands
            .push(Builtin::new(true_name, aliases, function))
    }

    // Finds a builtin command by name or alias
    // Returns None if the builtin does not exist
    fn resolve(&self, command_name: &str) -> Option<&Builtin> {
        for command in &self.commands {
            if command.true_name == command_name {
                return Some(command);
            }

            if command.aliases.contains(command_name) {
                return Some(command);
            }
        }

        None
    }

    // Evaluates and executes a command from a string
    pub fn eval(&self, context: &mut Context, line: &String) -> Result<()> {
        let commands = parser::parse(line);
        let mut results: Vec<Result<()>> = Vec::new();

        for (command_name, command_args) in commands {
            // ? Is there a way to avoid this type conversion?
            let command_name = command_name.as_str();
            let command_args = command_args.iter().map(|a| a.as_str()).collect();

            // Dispatch the command to the Dispatcher
            let result = self.dispatch(command_name, command_args, context);
            results.push(result);
        };

        for result in results {
             if result.is_err() {
                 return Err(result.err().unwrap())
             }
        };

        Ok(())
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
                Executable::new(path).run(context, command_args)
            } else {
                Err(DispatchError::UnknownCommand(command_name.to_string()).into())
            }
        }
    }
}
