use std::fs;
use std::env;

use colored::Colorize;

use crate::path::Path;
use crate::commands::{Context, StatusCode};

pub fn test(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("Test command!");
        StatusCode::success()
    } else {
        eprintln!("Usage: test");
        StatusCode::new(1)
    }
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        std::process::exit(0);
    } else {
        eprintln!("Usage: exit");
        StatusCode::new(1)
    }
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 0 {
        println!("{}", context.shell.environment().working_directory());
        StatusCode::success()
    } else {
        eprintln!("Usage: working-directory");
        StatusCode::new(1)
    }
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> StatusCode {
    if args.len() == 1 {
        match context
            .shell
            .environment()
            .working_directory_mut()
            .set_path(args[0])
        {
            true => {
                context.shell.environment().update_process_env_vars();
                StatusCode::success()
            }
            false => {
                eprintln!("Invalid path: '{}'", args[0]);
                StatusCode::new(1)
            }
        }
    } else {
        eprintln!("Usage: change-directory <path>");
        StatusCode::new(1)
    }
}

// TODO: Break up some of this code into different functions
pub fn list_files_and_directories(context: &mut Context, args: Vec<&str>) -> StatusCode {
    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid,
        // though in the future the error should be handled properly
        0 => fs::read_dir(env::current_dir().expect("Failed to get working directory"))
            .expect("Failed to read directory"),
        1 => {
            // Path::from_str_path() will attempt to expand and canonicalize the path, and return None if the path does not exist
            let absolute_path = match Path::from_str_path(args[0], context.shell.environment().home()) {
                Some(path) => path,
                None => {
                    eprintln!("Invalid path: '{}'", args[0]);
                    return StatusCode::new(2);
                }
            };

            match fs::read_dir(&absolute_path.absolute()) {
                Ok(files_and_directories) => files_and_directories,
                Err(_) => {
                    eprintln!("Failed to read directory: '{}'", absolute_path);
                    return StatusCode::new(3);
                }
            }
        }
        _ => {
            eprintln!("Usage: list-files-and-directories <path>");
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
            format!("{}/", fd_name).bright_green().to_string()
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
        eprintln!("Usage: clear-terminal");
        StatusCode::new(1)
    }
}

pub fn truncate(context: &mut Context, args: Vec<&str>) -> StatusCode {
    let truncation = match args.len() {
        0 => 1,
        // ! This is copilot code, it is extremely unsafe
        1 => args[0].parse::<usize>().unwrap(),
        _ => {
            eprintln!("Usage: truncate <length (default 1)>");
            return StatusCode::new(1);
        }
    };

    context
        .shell
        .environment()
        .working_directory_mut()
        .set_truncation(truncation);
    StatusCode::success()
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
        eprintln!("Usage: untruncate");
        StatusCode::new(1)
    }
}
