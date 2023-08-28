mod errors;
mod eval;
mod exec;
mod plugins;
mod state;

use anyhow::Result;

use errors::DispatchError;
use eval::{Dispatcher, LineEditor};
use plugins::host::PluginHost;
use state::shell::ShellState;

fn main() -> Result<()> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
    }));

    // The ShellState type stores all of the state for the shell, including its configuration,
    // its environment, and other miscellaneous data like command history
    let shell = ShellState::new()?;
    let plugins = PluginHost::new(shell.clone());
    // The LineEditor type is responsible for reading lines of input from the user, storing history,
    // providing tab completion and other line-editing features
    let mut line_editor = LineEditor::new();
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

// Prints an appropriate error message for the given error, if applicable
fn handle_error(error: Result<()>, shell: &mut ShellState) {
    match error {
        Ok(_) => shell.last_command_succeeded = true,
        Err(e) => {
            // TODO: Probably just do an enum here honestly
            match e.downcast_ref::<DispatchError>() {
                Some(DispatchError::UnknownCommand(command_name)) => {
                    println!("Unknown command: {}", command_name);
                }
                _ => {
                    if shell.config.show_errors {
                        // TODO: This is sort of a "magic" formatting string, it should be changed to a method or something
                        println!("Error: {:#?}: {}", e, e);
                    }
                }
            }

            shell.last_command_succeeded = false;
        }
    }
}
