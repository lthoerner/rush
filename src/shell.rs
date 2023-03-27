#![allow(dead_code, unused_variables)]

use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

use crate::path::Path;
use crate::commands::{CommandManager, Context};

pub struct Shell {
    user: String,
    cwd: Path,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            user: get_env_user(),
            cwd: Path::from_cwd(),
        }
    }

    // Repeatedly prompts the user for commands and executes them
    pub fn run(&mut self) {
        let user = get_env_user();
        let cwd_path = Path::from_cwd();

        // ? What should this name be?
        let dispatcher = CommandManager::default();
    
        loop {
            self.interpret(&dispatcher, self.prompt());
        }
    }

    // Displays the prompt and returns the user input
    fn prompt(&self) -> String {
        print!("{} on {} {} ", self.user.blue(), self.cwd.short().green(), ">".truecolor(60, 60, 60));
        flush();
        read_line()
    }

    // Interprets a command from a string
    fn interpret(&mut self, dispatcher: &CommandManager, line: String) {
        // Get the first word (the command name)
        let mut words = line.split_whitespace();
        let command_name = words.next().unwrap();

        // Get the rest of the words (the command arguments)
        let command_args: Vec<&str> = words.collect();

        // Bundle all the information that needs to be modifiable by the commands into a Context
        let mut context = Context::new(self);

        // Dispatch the command to the CommandManager
        let exit_code = dispatcher.dispatch(command_name, command_args, &mut context);

        // If the command was not found, print an error message
        if exit_code.is_none() {
            println!("{}: command not found", command_name.red());
        }
    }

    pub fn working_directory(&mut self) -> &mut Path {
        &mut self.cwd
    }
}

// Gets the name of the current user
fn get_env_user() -> String {
    var("USER").expect("Failed to get user")
}

// Flushes stdout
fn flush() {
    let mut stdout = stdout();
    stdout.flush().expect("Failed to flush");
}

// Reads a line of input from stdin
fn read_line() -> String {
    let mut line = String::new();
    let stdin = stdin();
    stdin.read_line(&mut line).expect("Failed to read line");

    line
}
