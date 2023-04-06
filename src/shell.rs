use std::io::{stdin, stdout, Write};

use anyhow::Result;
use colored::Colorize;

use crate::commands::{Context, Dispatcher};
use crate::environment::Environment;
use crate::errors::ShellError;

// Represents any settings for the shell, most of which can be configured by the user
pub struct Configuration {
    // The truncation length for the prompt
    truncation_factor: Option<usize>,
    // Whether or not to print out full error messages and status codes when a command fails
    show_errors: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            truncation_factor: None,
            show_errors: true,
        }
    }
}

impl Configuration {
    // Sets the truncation length for the prompt
    pub fn truncate(&mut self, length: usize) {
        self.truncation_factor = Some(length);
    }

    // Disables prompt truncation
    pub fn disable_truncation(&mut self) {
        self.truncation_factor = None;
    }

    // Enables or disables error messages
    pub fn show_errors(&mut self, show: bool) {
        self.show_errors = show;
    }
}

// Represents the shell, its state, and provides methods for interacting with it
pub struct Shell {
    pub environment: Environment,
    pub config: Configuration,
    success: bool,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Ok(Self {
            environment: Environment::new()?,
            config: Configuration::default(),
            success: true,
        })
    }

    // Repeatedly prompts the user for commands and executes them
    pub fn run(&mut self) -> Result<()> {
        let dispatcher = Dispatcher::default();

        loop {
            self.interpret(&dispatcher, self.prompt()?);
            // Print an extra line break to prevent malformed output
            println!();
        }
    }

    // Displays the prompt and returns the user input
    fn prompt(&self) -> Result<String> {
        let home = self.environment.home();
        print!(
            "{} on {}\n{} ",
            self.environment.user().blue(),
            self.environment
                .WORKING_DIRECTORY
                .collapse(home, self.config.truncation_factor)
                .green(),
            match self.success {
                true => "❯".bright_green().bold(),
                false => "❯".bright_red().bold(),
            }
        );

        flush()?;
        read_line()
    }

    // Interprets a command from a string
    fn interpret(&mut self, dispatcher: &Dispatcher, line: String) {
        let mut words = line.split_whitespace();
        // Get the first word (the command name)
        let command_name = words.next().unwrap();
        // Get the rest of the words (the command arguments)
        let command_args: Vec<&str> = words.collect();

        // Bundle all the information that needs to be modifiable by the commands into a Context
        let mut context = Context::new(self);

        // Dispatch the command to the Dispatcher
        let exit_code = dispatcher.dispatch(command_name, command_args, &mut context);

        // If the command was not found, print an error message
        match exit_code {
            Some(code) => {
                self.success = code.is_ok();
                if let Err(e) = code {
                    if self.config.show_errors {
                        eprintln!("Error: {}", format!("{:#?}: {}", e, e).red());
                    }
                }
            },
            None => {
                eprintln!("Unknown command: {}", command_name.red());
                self.success = false;
            }
        }
    }
}

// Flushes stdout
fn flush() -> Result<()> {
    let mut stdout = stdout();
    match stdout.flush() {
        Ok(_) => Ok(()),
        Err(_) => Err(ShellError::FailedToFlushStdout.into()),
    }
}

// Reads a line of input from stdin
fn read_line() -> Result<String> {
    let mut line = String::new();
    let stdin = stdin();
    match stdin.read_line(&mut line) {
        Ok(_) => (),
        Err(_) => return Err(ShellError::FailedToReadStdin.into()),
    }

    Ok(line)
}
