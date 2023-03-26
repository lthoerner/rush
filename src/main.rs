mod path;
mod prompt;
mod command;

use prompt::Prompt;

fn main() {
    let mut prompt = Prompt::new();
    prompt.run();
}
