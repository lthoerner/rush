/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

An 'External' will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use fs_err::{self, ReadDir};
use std::io::{BufRead, BufReader};

use anyhow::Result;

use rush_state::path::Path;
use rush_state::shell::Shell;
use rush_state::console::Console;

use crate::commands::{Executable, Runnable};
use crate::errors::BuiltinError;

pub fn test(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "test", console)?;
    console.println("Test command!");
    Ok(())
}

pub fn exit(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "exit", console)?;
    std::process::exit(0);
}

pub fn working_directory(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "working-directory", console)?;
    console.println(&format!("{}", shell.env().CWD()));
    Ok(())
}

pub fn change_directory(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "change-directory <path>", console)?;
    let history_limit = shell.config_mut().history_limit;
    shell
        .env_mut()
        .set_CWD(args[0], history_limit)
        .map_err(|_| {
            console.println(&format!("Invalid path: '{}'", args[0]));
            BuiltinError::FailedToRun.into()
        })
}

fn enter_and_read_path(shell: &mut Shell, console: &mut Console, path: &str) -> Result<ReadDir> {
    // Path::from_str() will attempt to expand and canonicalize the path, and return None if the path does not exist
    let absolute_path = Path::from_str(path, shell.env().HOME()).map_err(|_| {
        console.println(&format!("Invalid path: '{}'", path));
        BuiltinError::FailedToRun
    })?;

    Ok(fs_err::read_dir(absolute_path.path()).expect(&format!("Failed to read directory: '{}'", absolute_path)))
}

// TODO: Break up some of this code into different functions
pub fn list_directory(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    let show_hidden = show_hidden_files(&args);

    let files_and_directories = match args.len() {
        // Use the working directory as the default path argument
        // This uses expect() because it needs to crash if the working directory is invalid,
        // though in the future the error should be handled properly
        0 => fs_err::read_dir(shell.env().CWD().path())
            .expect("Failed to read directory"),
        1 => {
            if show_hidden  {
                fs_err::read_dir(shell.env().CWD().path()).expect("Failed to read directory")
            } else {
                enter_and_read_path(shell, console, args[0])?
            }
        }
        2 => {
            enter_and_read_path(shell, console, args[0])?
        }
        _ => {
            console.println("Usage: list-directory <path> [flags]");
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
        console.println(&directory);
    }

    for file in files {
        console.println(&file);
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
pub fn go_back(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-back", console)?;
    shell.env_mut().go_back().map_err(|_| {
        console.println("Previous directory does not exist or is invalid");
        BuiltinError::FailedToRun.into()
    })
}

pub fn go_forward(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-forward", console)?;
    shell.env_mut().go_forward().map_err(|_| {
        console.println("Next directory does not exist or is invalid");
        BuiltinError::FailedToRun.into()
    })
}

pub fn clear_terminal(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "clear-terminal", console)?;
    // * "Magic" ANSI escape sequence to clear the terminal
    console.println("\x1B[2J\x1B[1;1H");
    Ok(())
}

// TODO: Add prompt to confirm file overwrite
pub fn make_file(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::File::create(args[0]).map_err(|_| {
            console.println(&format!("Failed to create file: '{}'", args[0]));
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        console.println("Usage: make-file <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn make_directory(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::create_dir(args[0]).map_err(|_| {
            console.println(&format!("Failed to create directory: '{}'", args[0]));
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        console.println("Usage: make-directory <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn delete_file(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        fs_err::remove_file(args[0]).map_err(|_| {
            console.println(&format!("Failed to delete file: '{}'", args[0]));
            BuiltinError::FailedToRun
        })?;
        Ok(())
    } else {
        console.println("Usage: delete-file <path>");
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}

pub fn read_file(_shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "read-file <path>", console)?;
    let file_name = args[0].to_string();
    let file = fs_err::File::open(&file_name).map_err(|_| {
        console.println(&format!("Failed to open file: '{}'", file_name));
        BuiltinError::FailedToRun
    })?;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        console.println(&line);
    }

    Ok(())
}

pub fn run_executable(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "run-executable <path>", console)?;
    let executable_name = args[0].to_string();
    let executable_path = Path::from_str(&executable_name, shell.env().HOME()).map_err(|_| {
        console.println(&format!("Failed to resolve executable path: '{}'", executable_name));
        BuiltinError::FailedToRun
    })?;

    Executable::new(executable_path).run(shell, console, args)
}

pub fn configure(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "configure <key> <value>", console)?;
    let key = args[0];
    let value = args[1];

    match key {
        "truncation" => {
            if value == "false" {
                shell.config_mut().truncation_factor = None;
                return Ok(());
            }

            shell.config_mut().truncation_factor =
                Some(value.parse::<usize>().map_err(|_| {
                    console.println(&format!("Invalid truncation length: '{}'", value));
                    BuiltinError::InvalidValue(value.to_string())
                })?)
        }
        "history-limit" => {
            if value == "false" {
                shell.config_mut().history_limit = None;
                return Ok(());
            }

            shell.config_mut().history_limit =
                Some(value.parse::<usize>().map_err(|_| {
                    console.println(&format!("Invalid history limit: '{}'", value));
                    BuiltinError::InvalidValue(value.to_string())
                })?)
        }
        "show-errors" => {
            shell.config_mut().show_errors = value.parse::<bool>().map_err(|_| {
                console.println(&format!("Invalid value for show-errors: '{}'", value));
                BuiltinError::InvalidValue(value.to_string())
            })?
        }
        "multi-line-prompt" => {
            shell.config_mut().multi_line_prompt = value.parse::<bool>().map_err(|_| {
                console.println(&format!("Invalid value for multi-line-prompt: '{}'", value));
                BuiltinError::InvalidValue(value.to_string())
            })?
        }
        _ => {
            console.println(&format!("Invalid configuration key: '{}'", key));
            return Err(BuiltinError::InvalidArgument(key.to_string()).into());
        }
    }

    Ok(())
}

pub fn environment_variable(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "environment-variable <var>", console)?;
    match args[0].to_uppercase().as_str() {
        "PATH" => {
            for (i, path) in shell.env().PATH().iter().enumerate() {
                console.println(&format!("[{i}]: {path}"));
            }
        }
        "USER" => console.println(shell.env().USER()),
        "HOME" => console.println(&format!("{}", shell.env().HOME().display())),
        "CWD" | "WORKING-DIRECTORY" => console.println(&format!("{}", shell.env().CWD())),
        _ => {
            console.println(&format!("Invalid environment variable: '{}'", args[0]));
            return Err(BuiltinError::InvalidArgument(args[0].to_string()).into());
        }
    }

    Ok(())
}

pub fn edit_path(shell: &mut Shell, console: &mut Console, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "edit-path <append | prepend> <path>", console)?;
    let action = args[0];
    let path = Path::from_str(args[1], shell.env().HOME()).map_err(|_| {
        console.println(&format!("Invalid directory: '{}'", args[1]));
        BuiltinError::FailedToRun
    })?;

    match action {
        "append" => shell.env_mut().PATH_mut().push_front(path),
        "prepend" => shell.env_mut().PATH_mut().push_back(path),
        _ => {
            console.println(&format!("Invalid action: '{}'", action));
            return Err(BuiltinError::InvalidArgument(args[0].to_string()).into());
        }
    }

    Ok(())
}

// Convenience function for exiting a builtin on invalid argument count
fn check_args(args: &Vec<&str>, expected_args: usize, usage: &str, console: &mut Console) -> Result<()> {
    if args.len() == expected_args {
        Ok(())
    } else {
        console.println(&format!("Usage: {}", usage));
        Err(BuiltinError::InvalidArgumentCount(args.len()).into())
    }
}
