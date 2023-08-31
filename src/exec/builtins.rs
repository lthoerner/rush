/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

An executable will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use std::io::{stderr, BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{self, Clear, ClearType};

use super::builtin_arguments::ListDirectoryArguments;
use super::commands::{Executable, Runnable};
use crate::errors::{Handle, Result};
use crate::state::{Path, ShellState};

pub fn test(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "test")?;
    println!("{}", "Test command!".yellow());
    Ok(())
}

pub fn exit(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "exit")?;
    std::process::exit(0);
}

pub fn working_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "working-directory")?;
    println!("{}", shell.environment.CWD);
    Ok(())
}

pub fn change_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "change-directory <path>")?;
    let history_limit = shell.config.history_limit;
    shell
        .environment
        .set_CWD(args[0], history_limit)
        .replace_err_with_msg(
            builtin_err!(FailedToRun),
            &format!("Invalid path: '{}'", args[0]),
        )?;

    Ok(())
}

pub fn list_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = ListDirectoryArguments::parse_from(&args);
    let show_hidden = arguments.all;
    let path_to_read = match arguments.path {
        Some(path) => PathBuf::from(path),
        None => shell.environment.CWD.path().to_path_buf(),
    };

    let read_dir_result =
        fs_err::read_dir(&path_to_read).replace_err(path_err!(UnknownDirectory(path_to_read)))?;

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for dir_entry in read_dir_result {
        let fs_object = dir_entry.replace_err(builtin_err!(FailedReadingDir(path_to_read)))?;

        let fs_object_name = fs_object
            .file_name()
            .to_str()
            .replace_err(builtin_err!(FailedReadingFileName(path_to_read)))?;

        let fs_object_type = fs_object
            .file_type()
            .replace_err(builtin_err!(FailedReadingFileType(path_to_read)))?;

        if fs_object_name.starts_with('.') && !show_hidden {
            continue;
        }

        if fs_object_type.is_dir() {
            directories.push(format!("{}/", fs_object_name).green().to_string());
        } else {
            files.push(fs_object_name.cyan().to_string());
        };
    }

    directories.sort();
    files.sort();

    for directory in directories {
        println!("{}", &directory);
    }

    for file in files {
        println!("{}", &file);
    }

    Ok(())
}

pub fn previous_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-back")?;
    shell.environment.previous_directory().replace_err_with_msg(
        builtin_err!(FailedToRun),
        "Previous directory does not exist or is invalid",
    )
}

pub fn next_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "go-forward")?;
    shell.environment.next_directory().replace_err_with_msg(
        builtin_err!(FailedToRun),
        "Next directory does not exist or is invalid",
    )
}

pub fn clear_terminal(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 0, "clear-terminal")?;
    let y_size = terminal::size()
        .replace_err_with_msg(builtin_err!(FailedToRun), "Failed to get terminal size")?
        .1;

    execute!(stderr(), Clear(ClearType::All))
        .replace_err_with_msg(builtin_err!(FailedToRun), "Failed to clear terminal")?;

    execute!(stderr(), MoveTo(0, y_size - 2)).replace_err_with_msg(
        builtin_err!(FailedToRun),
        "Failed to move cursor to bottom of terminal",
    )
}

// TODO: Add prompt to confirm file overwrite
pub fn make_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "usage: make-file <path>")?;
    fs_err::File::create(args[0]).replace_err_with_msg(
        builtin_err!(FailedToRun),
        &format!("Failed to create file: '{}'", args[0]),
    )?;
    Ok(())
}

pub fn make_directory(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "make-directory <path>")?;
    fs_err::create_dir(args[0]).replace_err_with_msg(
        builtin_err!(FailedToRun),
        &format!("Failed to create directory: '{}'", args[0]),
    )
}

pub fn delete_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "delete-file <path>")?;
    fs_err::remove_file(args[0]).replace_err_with_msg(
        builtin_err!(FailedToRun),
        &format!("Failed to delete file: '{}'", args[0]),
    )
}

pub fn read_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "read-file <path>")?;
    let file_name = args[0].to_owned();
    let file = fs_err::File::open(&file_name).replace_err_with_msg(
        builtin_err!(FailedToRun),
        &format!("Failed to open file: '{}'", file_name),
    )?;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", &line);
    }

    Ok(())
}

pub fn run_executable(shell: &mut ShellState, mut args: Vec<&str>) -> Result<()> {
    let executable_name = args[0].to_owned();
    let executable_path = Path::from_str(&executable_name, &shell.environment.HOME)
        .replace_err_with_msg(
            builtin_err!(FailedToRun),
            &format!("Failed to resolve executable path: '{}'", executable_name),
        )?;

    // * Executable name is removed before running the executable because the std::process::Command
    // * process builder automatically adds the executable name as the first argument
    args.remove(0);
    Executable::new(executable_path).run(shell, args)
}

pub fn configure(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "configure <key> <value>")?;
    let key = args[0];
    let value = args[1];

    match key {
        "truncation" => {
            if value == "false" {
                shell.config.truncation_factor = None;
                return Ok(());
            }

            shell.config.truncation_factor = Some(value.parse::<usize>().replace_err_with_msg(
                builtin_err!(InvalidValue(value.to_owned())),
                &format!("Invalid truncation length: '{}'", value),
            )?);
        }
        "multi-line-prompt" => {
            shell.config.multi_line_prompt = value.parse::<bool>().replace_err_with_msg(
                builtin_err!(InvalidValue(value.to_owned())),
                &format!("Invalid value for multi-line-prompt: '{}'", value),
            )?;
        }
        "history-limit" => {
            if value == "false" {
                shell.config.history_limit = None;
                return Ok(());
            }

            shell.config.history_limit = Some(value.parse::<usize>().replace_err_with_msg(
                builtin_err!(InvalidValue(value.to_owned())),
                &format!("Invalid history limit: '{}'", value),
            )?);
        }
        "show-errors" => {
            shell.config.show_errors = value.parse::<bool>().replace_err_with_msg(
                builtin_err!(InvalidValue(value.to_owned())),
                &format!("Invalid value for show-errors: '{}'", value),
            )?;
        }
        _ => {
            return Err(builtin_err!(InvalidArgument(key.to_owned()))
                .set_context(&format!("Invalid configuration key: '{}'", key)));
        }
    }

    Ok(())
}

pub fn environment_variable(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 1, "environment-variable <var>")?;
    match args[0].to_uppercase().as_str() {
        "PATH" => {
            for (i, path) in shell.environment.PATH.iter().enumerate() {
                println!("[{i}]: {path}");
            }
        }
        "USER" => println!("{}", shell.environment.USER),
        "HOME" => println!("{}", shell.environment.HOME.display()),
        "CWD" | "WORKING-DIRECTORY" => println!("{}", shell.environment.CWD),
        _ => {
            return Err(builtin_err!(InvalidArgument(args[0].to_owned()))
                .set_context(&format!("Invalid environment variable: '{}'", args[0])));
        }
    }

    Ok(())
}

pub fn edit_path(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    check_args(&args, 2, "edit-path <append | prepend> <path>")?;
    let action = args[0];
    let path = Path::from_str(args[1], &shell.environment.HOME).replace_err_with_msg(
        builtin_err!(FailedToRun),
        &format!("Invalid directory: '{}'", args[1]),
    )?;

    match action {
        "append" => shell.environment.PATH.push_front(path),
        "prepend" => shell.environment.PATH.push_back(path),
        _ => {
            return Err(builtin_err!(InvalidArgument(action.to_owned()))
                .set_context(&format!("Invalid action: '{}'", action)));
        }
    }

    Ok(())
}

// Convenience function for exiting a builtin on invalid argument count
fn check_args(args: &Vec<&str>, expected_args: usize, usage: &str) -> Result<()> {
    if args.len() == expected_args {
        Ok(())
    } else {
        Err(
            builtin_err!(InvalidArgumentCount(expected_args, args.len()))
                .set_context(&format!("Usage: {}", usage)),
        )
    }
}
