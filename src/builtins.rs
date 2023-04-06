/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

You may notice that builtin commands are referenced in commands::Runnable::Internal. An 'Internal' is essentially a function pointer to a builtin command.
An 'External' will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use std::env;
use std::fs;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use colored::Colorize;

use crate::commands::{Context, Runnable};
use crate::errors::InternalCommandError;
use crate::path::Path;

pub fn test(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "test")?;
    println!("{}", "Test command!".yellow());
    Ok(())
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "exit")?;
    std::process::exit(0);
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "working-directory")?;
    println!("{}", context.cwd());
    Ok(())
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "change-directory <path>")?;
    context.env_mut().set_path(args[0]).map_err(|_| {
        eprintln!("Invalid path: '{}'", args[0]);
        InternalCommandError::FailedToRun.into()
    })
}

// TODO: Break up some of this code into different functions
pub fn list_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid,
        // though in the future the error should be handled properly
        0 => fs::read_dir(env::current_dir().expect("Failed to get working directory"))
            .expect("Failed to read directory"),
        1 => {
            // Path::from_str() will attempt to expand and canonicalize the path, and return None if the path does not exist
            let absolute_path = Path::from_str(args[0], context.home()).map_err(|_| {
                eprintln!("Invalid path: '{}'", args[0]);
                InternalCommandError::FailedToRun
            })?;

            fs::read_dir(&absolute_path.path()).map_err(|_| {
                eprintln!("Failed to read directory: '{}'", absolute_path.to_string());
                InternalCommandError::FailedToRun
            })?
        }
        _ => {
            eprintln!("Usage: list-directory <path>");
            return Err(InternalCommandError::InvalidArgumentCount.into());
        }
    };

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for fd in files_and_directories {
        let fd = fd.expect("Failed to read directory");

        let fd_name = fd
            .file_name()
            .to_str()
            .expect("Failed to read file name")
            .to_string();

        // TODO: Add a flag to show hidden files
        if fd_name.starts_with('.') {
            continue;
        }

        if fd.file_type().expect("Failed to read file type").is_dir() {
            // Append a '/' to directories
            let fd_name = format!("{}/", fd_name).bright_green().to_string();
            directories.push(fd_name)
        } else {
            files.push(fd_name)
        };
    }

    directories.sort();
    files.sort();

    for directory in directories {
        println!("{}", directory);
    }

    for file in files {
        println!("{}", file);
    }

    Ok(())
}

// TODO: Find a better name for this
pub fn go_back(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-back")?;
    context.env_mut().go_back().map_err(|_| {
        eprintln!("Previous directory does not exist or is invalid");
        InternalCommandError::FailedToRun.into()
    })
}

pub fn go_forward(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-forward")?;
    context.env_mut().go_forward().map_err(|_| {
        eprintln!("Next directory does not exist or is invalid");
        InternalCommandError::FailedToRun.into()
    })
}

pub fn clear_terminal(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "clear-terminal")?;
    // * "Magic" ANSI escape sequence to clear the terminal
    print!("\x1B[2J\x1B[1;1H");
    Ok(())
}

// TODO: Add prompt to confirm file overwrite
pub fn create_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs::File::create(args[0]).map_err(|_| {
            eprintln!("Failed to create file: '{}'", args[0]);
            InternalCommandError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: create-file <path>");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn create_directory(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs::create_dir(args[0]).map_err(|_| {
            eprintln!("Failed to create directory: '{}'", args[0]);
            InternalCommandError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: create-directory <path>");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn delete_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs::remove_file(args[0]).map_err(|_| {
            eprintln!("Failed to delete file: '{}'", args[0]);
            InternalCommandError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: delete-file <path>");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn read_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "read-file <path>")?;
    let file_name = args[0].to_string();
    let file = fs::File::open(&file_name).map_err(|_| {
        eprintln!("Failed to open file: '{}'", file_name);
        InternalCommandError::FailedToRun
    })?;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", line);
    }

    Ok(())
}

pub fn run_executable(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "run-executable <path>")?;
    let executable_name = args[0].to_string();
    let executable_path = Path::from_str(&executable_name, context.home()).map_err(|_| {
        eprintln!("Failed to resolve executable path: '{}'", executable_name);
        InternalCommandError::FailedToRun
    })?;

    Runnable::External(executable_path).run(context, args)
}

pub fn configure(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "configure <key> <value>")?;
    let key = args[0];
    let value = args[1];

    match key {
        "truncation" => {
            if value == "false" {
                context.shell_config().disable_truncation();
                return Ok(());
            }

            let truncation = value.parse::<usize>().map_err(|_| {
                eprintln!("Invalid truncation length: '{}'", value);
                InternalCommandError::InvalidValue
            })?;
            context.shell_config().truncate(truncation)
        }
        "show-errors" => {
            let show_errors = value.parse::<bool>().map_err(|_| {
                eprintln!("Invalid value for show-errors: '{}'", value);
                InternalCommandError::InvalidValue
            })?;
            context.shell_config().show_errors(show_errors)
        }
        "multi-line-prompt" => {
            let multi_line = value.parse::<bool>().map_err(|_| {
                eprintln!("Invalid value for multi-line-prompt: '{}'", value);
                InternalCommandError::InvalidValue
            })?;
            context.shell_config().multi_line_prompt(multi_line)
        }
        _ => {
            eprintln!("Invalid configuration key: '{}'", key);
            return Err(InternalCommandError::InvalidArgument.into());
        }
    }

    Ok(())
}

// Convenience function for exiting a builtin on invalid argument count
fn check_args(args: &Vec<&str>, expected_args: usize, usage: &str) -> Result<()> {
    if args.len() == expected_args {
        Ok(())
    } else {
        eprintln!("Usage: {}", usage);
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}
