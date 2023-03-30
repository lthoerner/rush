mod builtins;
mod commands;
mod environment;
mod path;
mod shell;
mod errors;

use shell::Shell;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
