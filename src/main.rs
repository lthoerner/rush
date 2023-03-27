mod path;
mod shell;
mod commands;
mod builtins;

use shell::Shell;

fn main() {
    let mut shell = Shell::new();
    shell.run();
}
