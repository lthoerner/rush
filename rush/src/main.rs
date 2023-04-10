use anyhow::Result;
use colored::Colorize;

use rush_eval::evaluator::Evaluator;
use rush_state::shell::Shell;
use rush_state::context::Context;
use rush_state::errors::ShellError;
use rush_console::reader::Console;
use rush_parser::tokenize;

fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    let mut console = Console::new();
    let evaluator = Evaluator::new();
    
    let mut context = Context::new(&mut shell.environment, &mut shell.config, &mut shell.success);

    loop {
        let line = console.read(&mut context)?;
        let (command_name, command_args) = tokenize(line);
        // ? Should this be done in the Console?
        match evaluator.eval(&mut context, command_name, command_args) {
            Ok(_) => context.set_command_success(true),
            Err(e) => {
                match e.downcast_ref::<ShellError>() {
                    Some(ShellError::UnknownCommand(command_name)) => {
                        eprintln!("Unknown command: {}", command_name.red());
                        context.set_command_success(false);
                    }
                    _ => {
                        if context.shell_config().show_errors {
                            eprintln!("Error: {}", format!("{:#?}: {}", e, e).red());
                        }
                    }
                }
            }
        }
    }
}
