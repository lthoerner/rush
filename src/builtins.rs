use crate::commands::{Context, StatusCode};

pub fn test(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("Test command!");
        StatusCode::success()
    } else {
        println!("Usage: test");
        StatusCode::new(1)
    }
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        std::process::exit(0);
    } else {
        println!("Usage: exit");
        StatusCode::new(1)
    }
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", context.shell.environment().working_directory());
        StatusCode::success()
    } else {
        println!("Usage: working-directory");
        StatusCode::new(1)
    }
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        let path = args[0];
        match context.shell.environment().working_directory_mut().set_path(path) {
            true => StatusCode::success(),
            false => {
                println!("Invalid path: {}", path);
                StatusCode::new(1)
            }
        }
    } else {
        println!("Usage: change-directory <path>");
        StatusCode::new(1)
    }
}

pub fn clear_terminal(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        // * "Magic" ANSI escape sequence to clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        StatusCode::success()
    } else {
        println!("Usage: clear-terminal");
        StatusCode::new(1)
    }
}

pub fn truncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    match args.len() {
        0 => {
            context
                .shell
                .environment()
                .working_directory_mut()
                .set_truncation(1);
            StatusCode::success()
        }
        1 => {
            // ! This is copilot code, it is probably extremely unsafe
            let truncation = args[0].parse::<usize>().unwrap();
            context
                .shell
                .environment()
                .working_directory_mut()
                .set_truncation(truncation);
            StatusCode::success()
        }
        _ => {
            println!("Usage: truncate <length (default 1)>");
            StatusCode::new(1)
        }
    }
}

pub fn untruncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        context
            .shell
            .environment()
            .working_directory_mut()
            .disable_truncation();
        StatusCode::success()
    } else {
        println!("Usage: untruncate");
        StatusCode::new(1)
    }
}
