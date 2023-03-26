#![allow(dead_code, unused_variables)]

use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

use crate::path::Path;
use crate::command::{Command, CommandManager};

// ? Should this have a different name?
pub struct Prompt {
    user: String,
    cwd: Path,
    commands: CommandManager,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            user: get_env_user(),
            cwd: Path::from_cwd(),
            commands: default_commands(),
        }
    }

    pub fn run(&mut self) {
        let user = get_env_user();
        let cwd_path = Path::from_cwd();
    
        loop {
            self.interpret(self.prompt());
        }
    }

    fn prompt(&self) -> String {
        print!("{} on {} {} ", self.user.blue(), self.cwd.short().green(), ">".truecolor(60, 60, 60));
        flush();
        read_line()
    }
    
    fn interpret(&mut self, line: String) {
        // TODO: Command resolution is messy due to using string lookup, find a different way
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

fn get_env_user() -> String {
    var("USER").expect("Failed to get user")
}

fn flush() {
    let mut stdout = stdout();
    stdout.flush().expect("Failed to flush");
}

fn read_line() -> String {
    let mut line = String::new();
    let stdin = stdin();
    stdin.read_line(&mut line).expect("Failed to read line");

    line
}

// TODO: Refactor this somehow
fn default_commands() -> CommandManager {
    let mut commands = CommandManager::new();

    // ? Should Command::new() be built into CommandManager::add_command()?
    commands.add_command(Command::new("exit", vec!["quit"]));
    commands.add_command(Command::new("test", vec![]));
    commands.add_command(Command::new("truncate", vec!["trunc"]));
    commands.add_command(Command::new("untruncate", vec!["untrunc"]));
    commands.add_command(Command::new("directory", vec!["dir", "pwd", "wd"]));

    commands
}
