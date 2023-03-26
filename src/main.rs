use std::io::{stdin, stdout, Write};

fn main() {
    loop {
        let line = prompt();
        print!("{}", line);
    }
}

fn prompt() -> String {
    print!("rush > ");
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
