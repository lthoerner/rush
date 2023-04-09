#![allow(unused_imports)]

use anyhow::Result;

use rush_eval::shell::Shell;
use rush_eval::commands::Context;
use rush_console::reader::Console;
use rush_parser::tokenize;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    let mut console = Console::new();
    
    loop {
        let mut context = Context::new(&mut shell.environment, &mut shell.config);
        let line = console.read(&mut context)?;
        let (command_name, command_args) = tokenize(line);
        shell.eval(command_name, command_args)?;
    }
}
