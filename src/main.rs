mod eval;
mod exec;
mod readline;
mod state;

use anyhow::Result;

use eval::dispatcher::Dispatcher;
use eval::errors::DispatchError;
use state::shell::ShellState;

fn main() -> Result<()> {
    // The Shell type stores all of the state for the shell, including its configuration,
    // its environment, and other miscellaneous data like command history
    let mut shell = ShellState::new()?;
    // The Console type is responsible for reading and writing to the terminal (TUI),
    // and providing an interface for any commands that need to produce output and/or take input
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_panic(info);
    }));
    // The Dispatcher type is responsible for resolving command names to actual function calls,
    // or executables if needed, and then invoking them with the given arguments
    let dispatcher = Dispatcher::default();

    loop {
        let line = readline::prompt_and_read_line();
        if let Some(line) = line {
            let status = dispatcher.eval(&mut shell, &line);
            handle_error(status, &mut shell);
        }
    }
}

// Prints an appropriate error message for the given error, if applicable
fn handle_error(error: Result<()>, shell: &mut ShellState) {
    match error {
        Ok(_) => shell.set_success(true),
        Err(e) => {
            match e.downcast_ref::<DispatchError>() {
                Some(DispatchError::UnknownCommand(command_name)) => {
                    println!("Unknown command: {}", command_name);
                }
                _ => {
                    if shell.config().show_errors {
                        // TODO: This is sort of a "magic" formatting string, it should be changed to a method or something
                        println!("Error: {:#?}: {}", e, e);
                    }
                }
            }

            shell.set_success(false);
        }
    }
}
