#![allow(unused_imports)]

use anyhow::Result;

use rush_shell::shell::Shell;
use rush_shell::commands::Context;
use rush_repl::prompt::Repl;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    let mut repl = Repl::new();
    
    loop {
        let mut context = Context::new(&mut shell.environment, &mut shell.config);
        let line = repl.read(&mut context)?;
        shell.eval(line)?;
    }
}