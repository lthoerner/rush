mod path;
mod shell;
mod command;

use shell::Shell;

fn main() {
    let mut prompt = Shell::new();
    prompt.run();
}
