use anyhow::Result;

use rush_shell::shell::Shell;
// use rush_repl::prompt::Repl;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new()?;
    shell.run()

    // let mut repl = Repl::new();
    // repl.run()
}