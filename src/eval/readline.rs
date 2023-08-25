use crossterm::style::Stylize;
use rustyline::error::ReadlineError;
use rustyline::{CompletionType, Config, DefaultEditor};

use crate::state::shell::ShellState;

pub struct LineEditor {
    editor: DefaultEditor,
}

impl LineEditor {
    // Creates a LineEditor with the default configuration and history file
    pub fn new() -> Self {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::Fuzzy)
            .build();

        // TODO: Make the history path a parameter
        let mut editor = DefaultEditor::with_config(config).unwrap();
        if editor.load_history("./config/history.rush").is_err() {
            println!("No existing history file found, attempting to create one...");
            if fs_err::File::create("./config/history.rush").is_err() {
                println!("Failed to create history file.");
            } else {
                println!("History file created successfully.");
                if editor.load_history("./config/history.rush").is_err() {
                    println!("Failed to load history file even though it exists.");
                }
            }
        }

        Self { editor }
    }

    pub fn prompt_and_read_line(&mut self, shell: &ShellState) -> Option<String> {
        loop {
            let input = self.editor.readline(&shell.generate_prompt());
            match input {
                Ok(line) => {
                    if !line.is_empty() {
                        // * This fails in the case of a blank/all-whitespace line,
                        // * a line that is already in the history, or if the history is full
                        // * None of these require any special handling
                        let _ = self.editor.add_history_entry(&line);
                        if self.editor.save_history("./config/history.rush").is_err() {
                            println!("Failed to save history file.");
                        }

                        return Some(line);
                    } else {
                        // TODO: Do not reprompt on a blank line
                        continue;
                    }
                }
                Err(e) => match e {
                    // TODO: Propagate error?
                    ReadlineError::Interrupted => std::process::exit(1),
                    ReadlineError::Eof => std::process::exit(0),
                    _ => {
                        println!("Unhandled error occurred while line-editing: {}", e);
                        std::process::exit(2);
                    }
                },
            }
        }
    }
}
