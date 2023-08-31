use std::os::unix::prelude::PermissionsExt;

use crate::exec::builtins;
use crate::exec::commands::{Builtin, Executable, Runnable};
use crate::state::{Path, ShellState};

use super::tokenizer::tokenize;
use crate::errors::Result;

/// Represents a collection of builtin commands
/// Allows for command resolution and execution through aliases
pub struct Dispatcher {
    commands: Vec<Builtin>,
}

impl Default for Dispatcher {
    /// Initializes the `Dispatcher` with the default shell commands and aliases
    #[rustfmt::skip]
    fn default() -> Self {
        let mut dispatcher = Self::new();

        dispatcher.add_builtin("test", vec!["t"], builtins::test);
        dispatcher.add_builtin("exit", vec!["quit", "q"], builtins::exit);
        dispatcher.add_builtin("working-directory", vec!["pwd", "wd"], builtins::working_directory);
        dispatcher.add_builtin("change-directory", vec!["cd"], builtins::change_directory);
        dispatcher.add_builtin("list-directory", vec!["directory", "list", "ls", "dir"], builtins::list_directory);
        dispatcher.add_builtin("previous-directory", vec!["back", "b", "prev", "pd"], builtins::previous_directory);
        dispatcher.add_builtin("next-directory", vec!["forward", "f", "next", "nd"], builtins::next_directory);
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

    /// Adds a builtin to the `Dispatcher`
    fn add_builtin<F: Fn(&mut ShellState, Vec<&str>) -> Result<()> + 'static>(
        &mut self,
        true_name: &str,
        aliases: Vec<&str>,
        function: F,
    ) {
        self.commands
            .push(Builtin::new(true_name, aliases, function))
    }

    /// Attempts to locate a builtin command by name or alias
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

    /// Evaluates and executes a command from a string
    pub fn eval(&self, shell: &mut ShellState, line: &str) -> Result<()> {
        let args = tokenize(line);
        let command_name = args.get(0).unwrap().as_str();
        let command_args: Vec<&str> = args.iter().skip(1).map(|a| a.as_str()).collect();
        self.dispatch(shell, command_name, command_args)?;

        Ok(())
    }

    /// Resolves and dispatches a command to the appropriate builtin or executable
    fn dispatch(
        &self,
        shell: &mut ShellState,
        command_name: &str,
        command_args: Vec<&str>,
    ) -> Result<()> {
        // If the command resides in the Dispatcher (generally means it is a builtin) run it
        if let Some(command) = self.resolve(command_name) {
            command.run(shell, command_args)
        } else {
            // If the command is not in the Dispatcher, try to run it as an executable from the PATH
            let path = Path::try_resolve_executable(command_name, &shell.environment.PATH);
            if let Ok(path) = path {
                // Check if the file is executable (has the executable bit set)
                if let Ok(metadata) = fs_err::metadata(path.path()) {
                    let permission_code = metadata.permissions().mode();
                    // 0o111 is the octal representation of 73, which is the executable bit
                    if permission_code & 0o111 == 0 {
                        Err(dispatch_err!(CommandNotExecutable(permission_code)))
                    } else {
                        Executable::new(path).run(shell, command_args)
                    }
                } else {
                    // If the file cannot be read, return an error
                    Err(dispatch_err!(FailedToReadExecutableMetadata(path.into())))
                }
            } else {
                Err(dispatch_err!(UnknownCommand(command_name.to_owned())))
            }
        }
    }
}
