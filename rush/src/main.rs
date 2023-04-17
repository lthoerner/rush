use anyhow::Result;

use rush_eval::dispatcher::Dispatcher;
use rush_eval::errors::DispatchError;
use rush_state::console::Console;
use rush_state::shell::Shell;

fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    let mut console = Console::new()?;
    let dispatcher = Dispatcher::default();
    
    console.enter()?;

    loop {
        let line = console.read_line(&mut shell)?;
        let status = dispatcher.eval(&mut shell, &mut console, &line);
        handle_error(status, &mut shell);
        
        if shell.success() {
            shell.history_add(line);
        }
    }
}

// Prints an appropriate error message for the given error, if applicable
fn handle_error(error: Result<()>, shell: &mut Shell) {
    match error {
        Ok(_) => shell.set_success(true),
        Err(e) => {
            match e.downcast_ref::<DispatchError>() {
                Some(DispatchError::UnknownCommand(command_name)) => {
                    eprintln!("Unknown command: {}", command_name);
                }
                _ => if shell.config().show_errors {
                    eprintln!("Error: {}", format!("{:#?}: {}", e, e));
                }
            }

            shell.set_success(false);
        },
    }
}
