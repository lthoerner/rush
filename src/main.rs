use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

fn main() {
    let user = get_env_user();
    let path = get_shortened_cwd();

    loop {
        let line = prompt(&user, &path);
        print!("{}", line);
    }
}

fn get_shortened_cwd() -> String {
    let full_path = get_env_cwd();
    let home_directory = get_env_home_directory();

    if full_path.starts_with(&home_directory) {
        let shortened_path = full_path.replace(&home_directory, "~");
        shortened_path
    } else {
        full_path
    }
}

fn get_env_user() -> String {
    var("USER").expect("Failed to get user")
}

fn get_env_cwd() -> String {
    var("PWD").expect("Failed to get path")
}

fn get_env_home_directory() -> String {
    var("HOME").expect("Failed to get home directory")
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
