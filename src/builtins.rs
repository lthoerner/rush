use crate::commands::{Context, StatusCode};

pub fn truncate(context: Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        // ! This is copilot code, it is probably extremely unsafe
        let truncation = args[0].parse::<usize>().unwrap();
        context.shell.working_directory().set_truncation(truncation);
        StatusCode::success()
    } else {
        println!("Usage: truncate <number of characters>");
        StatusCode::new(1)
    }
}

pub fn untruncate(context: Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        context.shell.working_directory().disable_truncation();
        StatusCode::success()
    } else {
        println!("Usage: untruncate");
        StatusCode::new(1)
    }
}

pub fn directory(context: Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", context.shell.working_directory());
        StatusCode::success()
    } else {
        println!("Usage: directory");
        StatusCode::new(1)
    }
}

pub fn exit(context: Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        std::process::exit(0);
    } else {
        println!("Usage: exit");
        StatusCode::new(1)
    }
}

pub fn test(context: Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("Test command!");
        StatusCode::success()
    } else {
        println!("Usage: test");
        StatusCode::new(1)
    }
}
