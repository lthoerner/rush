#![allow(dead_code, unused_variables)]

use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

use crate::path::Path;
use crate::command::CommandManager;

pub struct Shell {
    user: String,
    cwd: Path,
    commands: CommandManager,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            user: get_env_user(),
            cwd: Path::from_cwd(),
            commands: default_commands(),
        }
    }

    // Repeatedly prompts the user for commands and executes them
    pub fn run(&mut self) {
        let user = get_env_user();
        let cwd_path = Path::from_cwd();
    
        loop {
            self.interpret(self.prompt());
        }
    }

    // Displays the prompt and returns the user input
    fn prompt(&self) -> String {
        print!("{} on {} {} ", self.user.blue(), self.cwd.short().green(), ">".truecolor(60, 60, 60));
        flush();
        read_line()
    }
    
    // Interprets a command from a string
    fn interpret(&mut self, line: String) {
        if let Some(command) = self.commands.resolve(line.trim()) {
            match command.true_name().as_str() {
                "exit" => std::process::exit(0),
                "test" => println!("Test command!"),
                "truncate" => self.cwd.set_truncation(1),
                "untruncate" => self.cwd.disable_truncation(),
                "directory" => println!("{}", self.cwd),
                _ => panic!("Unexpected command"),
            }
        } else {
            println!("Unknown command");
        }
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

// Defines the commands that are available by default
// TODO: Refactor this somehow
fn default_commands() -> CommandManager {
    let mut commands = CommandManager::new();

    commands.add_command("exit", vec!["quit"]);
    commands.add_command("test", vec![]);
    commands.add_command("truncate", vec!["trunc"]);
    commands.add_command("untruncate", vec!["untrunc"]);
    commands.add_command("directory", vec!["dir", "pwd", "wd"]);

    commands
}
