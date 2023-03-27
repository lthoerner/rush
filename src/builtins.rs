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

pub fn directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", context.shell.working_directory());
        StatusCode::success()
    } else {
        println!("Usage: directory");
        StatusCode::new(1)
    }
}

pub fn truncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    match args.len() {
        0 => {
            context.shell.working_directory().set_truncation(1);
            StatusCode::success()
        }
        1 => {
            // ! This is copilot code, it is probably extremely unsafe
            let truncation = args[0].parse::<usize>().unwrap();
            context.shell.working_directory().set_truncation(truncation);
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
        context.shell.working_directory().disable_truncation();
        StatusCode::success()
    } else {
        println!("Usage: untruncate");
        StatusCode::new(1)
    }
}
