use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

use anyhow::Result;
use colored::Colorize;

use crate::commands::{Context, Dispatcher};
use crate::environment::Environment;
use crate::errors::ShellError;

// Represents any settings for the shell, most of which can be configured by the user
pub struct Configuration {
    // The truncation length for the prompt
    pub truncation_factor: Option<usize>,
    // How many directories to store in the back/forward history
    pub history_limit: Option<usize>,
    // Whether or not to print out full error messages and status codes when a command fails
    pub show_errors: bool,
    // Whether the prompt should be displayed in a single line or multiple lines
    pub multi_line_prompt: bool,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            truncation_factor: None,
            history_limit: None,
            show_errors: true,
            multi_line_prompt: false,
        }
    }
}

impl Configuration {
    fn from_file(filename: &str) -> Result<Self> {
        let mut config = Self::default();
        let file = File::open(filename).map_err(|_| ShellError::FailedToOpenConfigFile)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|_| ShellError::FailedToReadConfigFile)?;
            let tokens = line.split(": ").collect::<Vec<&str>>();
            if tokens.len() != 2 {
                return Err(ShellError::FailedToReadConfigFile.into());
            }

            let (key, value) = (tokens[0], tokens[1]);

            // ? Should these be underscores instead of hyphens?
            match key {
                "truncation-factor" => {
                    if let Ok(length) = value.parse::<usize>() {
                        config.truncation_factor = Some(length);
                    } else if value == "false" {
                        config.truncation_factor = None;
                    }
                }
                "history-limit" => {
                    if let Ok(limit) = value.parse::<usize>() {
                        config.history_limit = Some(limit);
                    } else if value == "false" {
                        config.history_limit = None;
                    }
                }
                "show-errors" => {
                    if let Ok(show) = value.parse::<bool>() {
                        config.show_errors = show;
                    }
                }
                "multi-line-prompt" => {
                    if let Ok(multi_line) = value.parse::<bool>() {
                        config.multi_line_prompt = multi_line;
                    }
                }
                _ => return Err(ShellError::FailedToReadConfigFile.into()),
            }
        }

        Ok(config)
    }
}

// Represents the shell, its state, and provides methods for interacting with it
pub struct Shell {
    pub environment: Environment,
    pub config: Configuration,
    dispatcher: Dispatcher,
    success: bool,
}

impl Shell {
    pub fn new() -> Result<Self> {
        Ok(Self {
            environment: Environment::new()?,
            config: Configuration::from_file("config/config.rush")?,
            dispatcher: Dispatcher::default(),
            success: true,
        })
    }

    // Evaluates and executes a command from a string
    // $ Somewhat temporary, probably will be combined with .interpret()
    pub fn eval(&mut self, line: String) -> Result<()> {
        self.interpret(line);
        Ok(())
    }

    // Interprets a command from a string
    fn interpret(&mut self, line: String) {
        // Determine the requested command and its arguments
        let (command_name, command_args) = split_arguments(&line);
        let command_name = command_name.as_str();
        let command_args = command_args.iter().map(|s| s.as_str()).collect();

        // Bundle all the information that needs to be modifiable by the commands into a Context
        let mut context = Context::new(&mut self.environment, &mut self.config);

        // Dispatch the command to the Dispatcher
        let exit_code = self.dispatcher.dispatch(command_name, command_args, &mut context);

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
// $ This is a temporary solution, and will be replaced by a proper parser
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