/*
A quick write-up on rush builtins:
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
    if args.len() == 0 {
        println!("{}", "Test command!".yellow());
        Ok(())
    } else {
        eprintln!("Usage: test");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn exit(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        std::process::exit(0);
    } else {
        eprintln!("Usage: exit");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn working_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        println!("{}", context.cwd());
        Ok(())
    } else {
        eprintln!("Usage: working-directory");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn change_directory(context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 1 {
        context.env_mut().set_path(args[0]).map_err(|_| {
            eprintln!("Invalid path: '{}'", args[0]);
            InternalCommandError::FailedToRun.into()
        })
    } else {
        eprintln!("Usage: change-directory <path>");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
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
    if args.len() == 0 {
        context.env_mut().go_back().map_err(|_| {
            eprintln!("Previous directory does not exist or is invalid");
            InternalCommandError::FailedToRun.into()
        })
    } else {
        eprintln!("Usage: go-back");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn go_forward(context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        context.env_mut().go_forward().map_err(|_| {
            eprintln!("Next directory does not exist or is invalid");
            InternalCommandError::FailedToRun.into()
        })
    } else {
        eprintln!("Usage: go-forward");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

pub fn clear_terminal(_context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        // * "Magic" ANSI escape sequence to clear the terminal
        print!("\x1B[2J\x1B[1;1H");
        Ok(())
    } else {
        eprintln!("Usage: clear-terminal");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
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
    let file_name = match args.len() {
        1 => args[0].to_string(),
        _ => {
            eprintln!("Usage: read-file <path>");
            return Err(InternalCommandError::InvalidArgumentCount.into());
        }
    };

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
    let executable_name = match args.len() {
        1 => args[0].to_string(),
        _ => {
            eprintln!("Usage: run-executable <path>");
            return Err(InternalCommandError::InvalidArgumentCount.into());
        }
    };

    let executable_path = Path::from_str(&executable_name, context.home()).map_err(|_| {
        eprintln!("Failed to resolve executable path: '{}'", executable_name);
        InternalCommandError::FailedToRun
    })?;

    Runnable::External(executable_path).run(context, args)
}

// TODO: Move truncate() and untruncate() to a general shell configuration command
pub fn truncate(context: &mut Context, args: Vec<&str>) -> Result<()> {
    let truncation = match args.len() {
        0 => 1,
        1 => args[0].parse::<usize>().map_err(|_| {
            eprintln!("Invalid truncation length: '{}'", args[0]);
            InternalCommandError::InvalidValue
        })?,
        _ => {
            eprintln!("Usage: truncate <length (default 1)>");
            return Err(InternalCommandError::InvalidArgumentCount.into());
        }
    };

    Ok(context.shell_config().truncate(truncation))
}

pub fn untruncate(context: &mut Context, args: Vec<&str>) -> Result<()> {
    if args.len() == 0 {
        Ok(context.shell_config().disable_truncation())
    } else {
        eprintln!("Usage: untruncate");
        Err(InternalCommandError::InvalidArgumentCount.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Shell;

    #[test]
    fn test_command_test_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = test(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_exit_success() {
        // * This is a placeholder test because the exit command
        // * will exit the program, effectively ending the test
    }

    #[test]
    fn test_command_working_directory_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = working_directory(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_change_directory_success_1() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = change_directory(&mut context, vec!["/"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_change_directory_success_2() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = change_directory(&mut context, vec!["~"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_change_directory_success_3() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        change_directory(&mut context, vec!["~"]).unwrap();
        // ! This is not guaranteed to exist on the tester's system
        let status = change_directory(&mut context, vec!["Documents"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_change_directory_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = change_directory(&mut context, vec!["/invalid/path"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_list_directory_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = list_directory(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_list_directory_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = list_directory(&mut context, vec!["/invalid/path"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_go_back_success() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        context.env_mut().set_path("/").unwrap();
        let status = go_back(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_go_back_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = go_back(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_truncate_success_1() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = truncate(&mut context, Vec::new());
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_truncate_success_2() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = truncate(&mut context, vec!["10"]);
        assert!(status.is_ok());
    }

    #[test]
    fn test_command_truncate_fail() {
        let mut shell = Shell::new().unwrap();
        let mut context = Context::new(&mut shell);
        let status = truncate(&mut context, vec!["-10"]);
        assert!(status.is_ok());
    }
}
