mod path;

use std::io::{stdin, stdout, Write};
use std::env::var;

use colored::Colorize;

use path::Path;

fn main() {
    let user = get_env_user();
    let cwd_path = Path::from_cwd();

    loop {
        let line = prompt(&user, &cwd_path);
        print!("{}", line);
    }
}

fn get_env_user() -> String {
    var("USER").expect("Failed to get user")
}

fn prompt(user: &String, path: &Path) -> String {
    print!("{} on {} > ", user.blue(), path.short().green());
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
