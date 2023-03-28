use std::fs;
use std::path::PathBuf;

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
        match context
            .shell
            .environment()
            .working_directory_mut()
            .set_path(path)
        {
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

pub fn list_files_and_directories(context: &mut Context, args: Vec<&str>) -> StatusCode {
    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid
        0 => fs::read_dir(context.shell.environment().working_directory().absolute())
            .expect("Failed to read directory"),
        1 => {
            let path = PathBuf::from(args[0]);

            if !path.exists() {
                println!("Invalid path: {}", path.to_string_lossy().to_string());
                return StatusCode::new(2);
            }

            match fs::read_dir(&path) {
                Ok(files_and_directories) => files_and_directories,
                Err(_) => {
                    println!(
                        "Failed to read directory: {}",
                        path.to_string_lossy().to_string()
                    );
                    return StatusCode::new(3);
                }
            }
        }
        _ => {
            println!("Usage: list-files-and-directories");
            return StatusCode::new(1);
        }
    };

    for fd in files_and_directories {
        let fd = fd.expect("Failed to read directory");
        let fd_name = fd
            .file_name()
            .to_str()
            .expect("Failed to read file name")
            .to_string();

        // Append a '/' to directories
        let fd = if fd.file_type().expect("Failed to read file type").is_dir() {
            format!("{}/", fd_name)
        } else {
            fd_name
        };

        println!("{}", fd);
    }

    StatusCode::success()
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
