#![allow(unused_imports)]

use anyhow::Result;

use rush_shell::shell::Shell;
use rush_shell::commands::Context;
use rush_repl::prompt::Repl;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    shell.run()?;
    // let mut repl = Repl::new();
    // let mut context = Context::new(&mut shell);

    // repl.read(&mut context)?;

    Ok(())
}