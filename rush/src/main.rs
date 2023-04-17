use anyhow::Result;

use rush_eval::dispatcher::Dispatcher;
use rush_eval::errors::DispatchError;
use rush_state::console::Console;
use rush_state::shell::Shell;

fn main() -> Result<()> {
    // The Shell type stores all of the state for the shell, including its configuration,
    // its environment, and other miscellaneous data like command history
    let mut shell = Shell::new()?;
    // The Console type is responsible for reading and writing to the terminal (TUI),
    // and providing an interface for any commands that need to produce output and/or take input
    let mut console = Console::new()?;
    // The Dispatcher type is responsible for resolving command names to actual function calls,
    // or executables if needed, and then invoking them with the given arguments
    let dispatcher = Dispatcher::default();
    
    console.enter()?;

    loop {
        let line = console.read_line(&mut shell)?;
        let status = dispatcher.eval(&mut shell, &mut console, &line);
        handle_error(status, &mut shell, &mut console);
        
        if shell.success() {
            shell.history_add(line);
        }
    }
}

// Prints an appropriate error message for the given error, if applicable
fn handle_error(error: Result<()>, shell: &mut Shell, console: &mut Console) {
    match error {
        Ok(_) => shell.set_success(true),
        Err(e) => {
            match e.downcast_ref::<DispatchError>() {
                Some(DispatchError::UnknownCommand(command_name)) => {
                    console.println(&format!("Unknown command: {}", command_name));
                }
                _ => if shell.config().show_errors {
                    // TODO: This is sort of a "magic" formatting string, it should be changed to a method or something
                    console.println(&format!("Error: {}", format!("{:#?}: {}", e, e)));
                }
            }

            shell.set_success(false);
        },
    }
}
