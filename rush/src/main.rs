use anyhow::Result;
use colored::Colorize;

use rush_console::reader::Console;
use rush_eval::dispatcher::Dispatcher;
use rush_state::errors::ShellError;
use rush_state::shell::Context;
use rush_state::shell::Shell;

fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    let mut console = Console::new();
    let dispatcher = Dispatcher::default();

    let mut context = Context::new(&mut shell);

    loop {
        let line = console.read(&mut context)?;
        let status = dispatcher.eval(&mut context, line);
        // ? Should this be done in the Console?
        handle_error(status, &mut context);
    }
}

// Prints an appropriate error message for the given error, if applicable
fn handle_error(error: Result<()>, context: &mut Context) {
    match error {
        Ok(_) => context.set_success(true),
        Err(e) => match e.downcast_ref::<ShellError>() {
            Some(ShellError::UnknownCommand(command_name)) => {
                eprintln!("Unknown command: {}", command_name.red());
                context.set_success(false);
            }
            _ => {
                if context.shell_config().show_errors {
                    eprintln!("Error: {}", format!("{:#?}: {}", e, e).red());
                }
            }
        },
    }
}
