mod builtins;
mod commands;
mod environment;
mod errors;
mod path;
mod shell;

use anyhow::Result;

use shell::Shell;

// TODO: Add upstream error handling here
fn main() -> Result<()> {
    let mut shell = Shell::new();
    shell.run()
}
