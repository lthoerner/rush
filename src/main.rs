mod path;
mod prompt;

use prompt::Prompt;

fn main() {
    let mut prompt = Prompt::new();
    prompt.run();
}
