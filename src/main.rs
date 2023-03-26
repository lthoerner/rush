use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

fn main() {
    let user = get_current_user();
    let path = get_current_path();

    loop {
        let line = prompt(&user, &path);
        print!("{}", line);
    }
}

fn get_current_user() -> String {
    var("USER").expect("Failed to get user")
}

fn get_current_path() -> String {
    var("PWD").expect("Failed to get path")
}

fn prompt(user: &String, path: &String) -> String {
    print!("{} on {} > ", user.blue(), path.green());
    flush();
    read_line()
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
