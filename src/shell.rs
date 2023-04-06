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
    // Whether the prompt should be displayed in a single line or multiple lines
    multi_line_prompt: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            truncation_factor: None,
            show_errors: true,
            multi_line_prompt: false,
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

    // Enables or disables multi-line prompts
    pub fn multi_line_prompt(&mut self, multi_line: bool) {
        self.multi_line_prompt = multi_line;
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
        let home = self.environment.HOME();
        print!(
            "{} on {}{}{} ",
            self.environment.USER().blue(),
            self.environment
                .WORKING_DIRECTORY
                .collapse(home, self.config.truncation_factor)
                .green(),
            match self.config.multi_line_prompt {
                true => "\n",
                false => " ",
            },
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
        // Determine the requested command and its arguments
        let (command_name, command_args) = split_arguments(&line);
        let command_name = command_name.as_str();
        let command_args = command_args.iter().map(|s| s.as_str()).collect();

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
            }
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

// Splits arguments by spaces, taking quotes into account
// ! This is a temporary solution, and will be replaced by a proper parser
fn split_arguments(line: &str) -> (String, Vec<String>) {
    let line = line.trim();
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        match c {
            '"' => {
                if in_quotes {
                    args.push(current_arg);
                    current_arg = String::new();
                }

                in_quotes = !in_quotes;
            }
            ' ' => {
                if in_quotes {
                    current_arg.push(c);
                } else {
                    args.push(current_arg);
                    current_arg = String::new();
                }
            }
            _ => current_arg.push(c),
        }
    }

    if args.is_empty() {
        return (current_arg, Vec::new());
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    (args.remove(0), args)
}
