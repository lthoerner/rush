#![allow(dead_code, unused_variables)]

use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

use crate::path::Path;

// ? Should this have a different name?
pub struct Prompt {
    user: String,
    cwd: Path,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            user: get_env_user(),
            cwd: Path::from_cwd(),
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
        print!("{} on {} > ", self.user.blue(), self.cwd.short().green());
        flush();
        read_line()
    }
    
    fn interpret(&mut self, line: String) {
        let line = line.trim();
        match line {
            "exit" => std::process::exit(0),
            "truncate" => self.cwd.set_truncation(1),
            "untruncate" => self.cwd.disable_truncation(),
            _ => println!("Unknown command"),
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
