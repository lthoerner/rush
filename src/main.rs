mod builtins;
mod commands;
mod environment;
mod path;
mod shell;
mod errors;

use anyhow::Result;

use shell::Shell;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
