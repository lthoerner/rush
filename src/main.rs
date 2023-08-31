#[macro_use]
mod errors;
mod eval;
mod exec;
mod plugins;
mod state;

use errors::{Result, RushError};
use eval::{Dispatcher, LineEditor};
use plugins::host::PluginHost;
use state::ShellState;

fn main() {
    // The ShellState type stores all of the state for the shell, including its configuration,
    // its environment, and other miscellaneous data like command history
    let Ok(shell) = ShellState::new() else {
        std::process::exit(1);
    };

    let plugins = PluginHost::new(shell.clone());

    // The LineEditor type is responsible for reading lines of input from the user, storing history,
    // providing tab completion and other line-editing features
    let mut line_editor = match LineEditor::new("./config/history.rush") {
        Ok(editor) => editor,
        Err(err) => crash_with_error(err),
    };

    // The Dispatcher type is responsible for resolving command names to actual function calls,
    // or executables if needed, and then invoking them with the given arguments
    let dispatcher = Dispatcher::default();

    loop {
        let line = line_editor.prompt_and_read_line(&shell.read().unwrap());
        if let Some(line) = line {
            let status = dispatcher.eval(&mut shell.write().unwrap(), &line);
            handle_error(status, &mut shell.write().unwrap());
        }
    }
}

// Handles the return value of running a builtin or executable, setting flags and/or printing errors
fn handle_error(potential_error: Result<()>, shell: &mut ShellState) {
    shell.last_command_succeeded = potential_error.is_ok();
    if let Err(error) = potential_error {
        eprintln!("{}", error);
    }
}

// Handles errors from which the shell cannot recover, mainly errors arising from shell setup
fn crash_with_error(error: RushError) -> ! {
    eprintln!("{}", error);
    // TODO: Maybe return status code for each error type?
    std::process::exit(1);
}
