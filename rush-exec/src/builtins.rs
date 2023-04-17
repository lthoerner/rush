/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

An 'External' will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use fs_err;
use std::env;
use std::io::{BufRead, BufReader};

use anyhow::Result;

use rush_state::path::Path;
use rush_state::context::Context;

use crate::commands::{Executable, Runnable};
use crate::errors::BuiltinError;

pub fn test(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "test")?;
    println!("{}", "Test command!");
    Ok(())
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "exit")?;
    std::process::exit(0);
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "working-directory")?;
    println!("{}", context.env().CWD());
    Ok(())
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "change-directory <path>")?;
    let history_limit = context.shell_config_mut().history_limit;
    context
        .env_mut()
        .set_CWD(args[0], history_limit)
        .map_err(|_| {
            eprintln!("Invalid path: '{}'", args[0]);
            BuiltinError::FailedToRun.into()
        })
}

fn enter_and_read_path(
    context: &mut Context,
    path: &str,
) -> Result<fs_err::ReadDir, BuiltinError> {
    // Path::from_str() will attempt to expand and canonicalize the path, and return None if the path does not exist
    let absolute_path = Path::from_str(path, context.env().HOME()).map_err(|_| {
        eprintln!("Invalid path: '{}'", path);
        BuiltinError::FailedToRun
    })?;

    fs_err::read_dir(&absolute_path.path()).map_err(|_| {
        eprintln!("Failed to read directory: '{}'", absolute_path.to_string());
        BuiltinError::FailedToRun
    })
}

// TODO: Break up some of this code into different functions
pub fn list_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    let show_hidden = show_hidden_files(&args);

    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid,
        // though in the future the error should be handled properly
        0 => fs_err::read_dir(env::current_dir().expect("Failed to get working directory"))
            .expect("Failed to read directory"),
        1 => {
            if show_hidden  {
                fs_err::read_dir(env::current_dir().expect("Failed to get working directory"))
                    .expect("Failed to read directory")
            } else {
                enter_and_read_path(context, &args[0])?
            }
        }
        2 => {
            enter_and_read_path(context, &args[0])?
        }
        _ => {
            eprintln!("Usage: list-directory <path> [flags]");
            return Err(BuiltinError::InvalidArgumentCount(args.len()).into());
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

        if fd_name.starts_with('.') && !show_hidden {
            continue;
        }

        if fd.file_type().expect("Failed to read file type").is_dir() {
            // Append a '/' to directories
            let fd_name = format!("{}/", fd_name).to_string();
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

fn show_hidden_files(args: &Vec<&str>) -> bool {
    let show_all_flags = vec!["--show-hidden", "--all", "-a"];

    if args.len() == 2 {
        show_all_flags.contains(&args[1])
    } else if args.len() == 1 {
        show_all_flags.contains(&args[0])
    } else {
        false
    }
}

// TODO: Find a better name for this
pub fn go_back(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-back")?;
    context.env_mut().go_back().map_err(|_| {
        eprintln!("Previous directory does not exist or is invalid");
        BuiltinError::FailedToRun.into()
    })
}

pub fn go_forward(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-forward")?;
    context.env_mut().go_forward().map_err(|_| {
        eprintln!("Next directory does not exist or is invalid");
        BuiltinError::FailedToRun.into()
    })
}

pub fn clear_terminal(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "clear-terminal")?;
    // * "Magic" ANSI escape sequence to clear the terminal
    print!("\x1B[2J\x1B[1;1H");
    Ok(())
}

// TODO: Add prompt to confirm file overwrite
pub fn make_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::File::create(args[0]).map_err(|_| {
            eprintln!("Failed to create file: '{}'", args[0]);
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: make-file <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn make_directory(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::create_dir(args[0]).map_err(|_| {
            eprintln!("Failed to create directory: '{}'", args[0]);
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: make-directory <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn delete_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::remove_file(args[0]).map_err(|_| {
            eprintln!("Failed to delete file: '{}'", args[0]);
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        eprintln!("Usage: delete-file <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn read_file(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "read-file <path>")?;
    let file_name = args[0].to_string();
    let file = fs_err::File::open(&file_name).map_err(|_| {
        eprintln!("Failed to open file: '{}'", file_name);
        BuiltinError::FailedToRun
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
    let executable_path = Path::from_str(&executable_name, context.env().HOME()).map_err(|_| {
        eprintln!("Failed to resolve executable path: '{}'", executable_name);
        BuiltinError::FailedToRun
    })?;

    Executable::new(executable_path).run(context, args)
}

pub fn configure(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "configure <key> <value>")?;
    let key = args[0];
    let value = args[1];

    match key {
        "truncation" => {
            if value == "false" {
                context.shell_config_mut().truncation_factor = None;
                return Ok(());
            }

            context.shell_config_mut().truncation_factor =
                Some(value.parse::<usize>().map_err(|_| {
                    eprintln!("Invalid truncation length: '{}'", value);
                    BuiltinError::InvalidValue(value.to_string())
                })?)
        }
        "history-limit" => {
            if value == "false" {
                context.shell_config_mut().history_limit = None;
                return Ok(());
            }

            context.shell_config_mut().history_limit =
                Some(value.parse::<usize>().map_err(|_| {
                    eprintln!("Invalid history limit: '{}'", value);
                    BuiltinError::InvalidValue(value.to_string())
                })?)
        }
        "show-errors" => {
            context.shell_config_mut().show_errors = value.parse::<bool>().map_err(|_| {
                eprintln!("Invalid value for show-errors: '{}'", value);
                BuiltinError::InvalidValue(value.to_string())
            })?
        }
        "multi-line-prompt" => {
            context.shell_config_mut().multi_line_prompt = value.parse::<bool>().map_err(|_| {
                eprintln!("Invalid value for multi-line-prompt: '{}'", value);
                BuiltinError::InvalidValue(value.to_string())
            })?
        }
        _ => {
            eprintln!("Invalid configuration key: '{}'", key);
            return Err(BuiltinError::InvalidArgument(key.to_string()).into());
        }
    }

    Ok(())
}

pub fn environment_variable(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "environment-variable <var>")?;
    match args[0].to_uppercase().as_str() {
        "PATH" => {
            for (i, path) in context.env().PATH().iter().enumerate() {
                println!("[{i}]: {path}");
            }
        }
        "USER" => println!("{}", context.env().USER()),
        "HOME" => println!("{}", context.env().HOME().display()),
        "CWD" | "WORKING-DIRECTORY" => println!("{}", context.env().CWD()),
        _ => {
            eprintln!("Invalid environment variable: '{}'", args[0]);
            return Err(BuiltinError::InvalidArgument(args[0].to_string()).into());
        }
    }

    Ok(())
}

pub fn edit_path(context: &mut Context, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "edit-path <append | prepend> <path>")?;
    let action = args[0];
    let path = Path::from_str(args[1], context.env().HOME()).map_err(|_| {
        eprintln!("Invalid directory: '{}'", args[1]);
        BuiltinError::FailedToRun
    })?;

    match action {
        "append" => context.env_mut().PATH_mut().push_front(path),
        "prepend" => context.env_mut().PATH_mut().push_back(path),
        _ => {
            eprintln!("Invalid action: '{}'", action);
            return Err(BuiltinError::InvalidArgument(args[0].to_string()).into());
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
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}
