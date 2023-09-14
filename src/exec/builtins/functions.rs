/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

An executable will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use std::io::{stderr, BufRead, BufReader};

use clap::Parser;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::Stylize;
use crossterm::terminal::{self, Clear, ClearType};

use super::args::{
    ChangeDirectoryArgs, ClearTerminalArgs, ConfigureArgs, DeleteFileArgs, EditPathArgs,
    EditPathSubcommand, EnvironmentVariableArgs, ExitArgs, ListDirectoryArgs, MakeDirectoryArgs,
    MakeFileArgs, NextDirectoryArgs, PreviousDirectoryArgs, ReadFileArgs, RunExecutableArgs,
    WorkingDirectoryArgs,
};
use crate::errors::{Handle, Result};
use crate::exec::builtins::args::{
    AppendPathCommand, DeletePathCommand, InsertPathCommand, PrependPathCommand, TestArgs,
};
use crate::exec::{Executable, Runnable};
use crate::state::{EnvVariable, Path, ShellState};

pub fn test(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    clap_handle!(TestArgs::try_parse_from(args));
    println!("{}", "Test command!".yellow());
    Ok(())
}

pub fn exit(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    clap_handle!(ExitArgs::try_parse_from(args));
    std::process::exit(0);
}

pub fn working_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    clap_handle!(WorkingDirectoryArgs::try_parse_from(args));
    println!("{}", shell.CWD());
    Ok(())
}

pub fn change_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(ChangeDirectoryArgs::try_parse_from(args));
    let history_limit = shell.config.history_limit;
    shell
        .environment
        .set_CWD(&arguments.path, history_limit)
        .replace_err(|| file_err!(UnknownPath: arguments.path))?;

    Ok(())
}

pub fn list_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(ListDirectoryArgs::try_parse_from(&args));
    let show_hidden = arguments.show_hidden;
    let path_to_read = arguments.path.unwrap_or(shell.CWD().path().to_path_buf());

    let read_dir_result =
        fs_err::read_dir(&path_to_read).replace_err(|| file_err!(UnknownPath: path_to_read))?;

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for dir_entry in read_dir_result {
        let fs_object = dir_entry.replace_err(|| file_err!(UnreadableDirectory: path_to_read))?;
        let fs_object_name = fs_object.file_name();
        let fs_object_name = fs_object_name
            .to_str()
            .replace_err(|| file_err!(UnreadableFileName: path_to_read))?;

        let fs_object_type = fs_object
            .file_type()
            .replace_err(|| file_err!(UnreadableFileType: path_to_read))?;

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
    clap_handle!(PreviousDirectoryArgs::try_parse_from(args));
    shell
        .environment
        .previous_directory()
        .replace_err(|| state_err!(NoPreviousDirectory))
}

pub fn next_directory(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    clap_handle!(NextDirectoryArgs::try_parse_from(args));
    shell
        .environment
        .next_directory()
        .replace_err(|| state_err!(NoNextDirectory))
}

pub fn clear_terminal(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    clap_handle!(ClearTerminalArgs::try_parse_from(args));
    let y_size = terminal::size()
        .replace_err_with_msg(
            || builtin_err!(TerminalOperationFailed),
            "Could not get terminal size",
        )?
        .1;

    execute!(stderr(), Clear(ClearType::All)).replace_err_with_msg(
        || builtin_err!(TerminalOperationFailed),
        "Could not clear terminal",
    )?;

    execute!(stderr(), MoveTo(0, y_size - 2)).replace_err_with_msg(
        || builtin_err!(TerminalOperationFailed),
        "Could not move cursor to bottom of terminal",
    )
}

// TODO: Add prompt to confirm file overwrite
pub fn make_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(MakeFileArgs::try_parse_from(args));
    fs_err::File::create(&arguments.path)
        .replace_err(|| file_err!(CouldNotCreateFile: arguments.path))?;
    Ok(())
}

pub fn make_directory(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(MakeDirectoryArgs::try_parse_from(args));
    fs_err::create_dir(&arguments.path)
        .replace_err(|| file_err!(CouldNotCreateDirectory: arguments.path))
}

pub fn delete_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(DeleteFileArgs::try_parse_from(args));
    fs_err::remove_file(&arguments.path)
        .replace_err(|| file_err!(CouldNotDeleteFile: arguments.path))
}

pub fn read_file(_shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(ReadFileArgs::try_parse_from(args));
    let file_name = arguments.path;
    let file =
        fs_err::File::open(&file_name).replace_err(|| file_err!(CouldNotOpenFile: file_name))?;

    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", &line);
    }

    Ok(())
}

pub fn run_executable(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(RunExecutableArgs::try_parse_from(&args));
    let executable_name = arguments.path;
    let executable_path = Path::try_from_path(&executable_name, Some(&shell.environment.HOME))
        .replace_err_with_msg(
            || file_err!(UnknownPath: executable_name),
            &format!("Could not find executable '{}'", executable_name.display()),
        )?;

    // TODO: Fix the usage of args and arg parsing here
    Executable::new(executable_path).run(shell, args)
}

pub fn configure(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(ConfigureArgs::try_parse_from(args));

    if let Some(truncation) = arguments.truncation {
        shell.config.truncation = truncation.into();
    }

    if let Some(history_limit) = arguments.history_limit {
        shell.config.history_limit = history_limit.into();
    }

    if let Some(multiline_prompt) = arguments.multiline_prompt {
        shell.config.multiline_prompt = multiline_prompt.into();
    }

    if let Some(show_errors) = arguments.show_errors {
        shell.config.show_errors = show_errors.into();
    }

    Ok(())
}

pub fn environment_variable(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(EnvironmentVariableArgs::try_parse_from(args));
    use EnvVariable::*;
    match arguments.variable {
        USER => println!("{}", shell.environment.USER),
        HOME => println!("{}", shell.environment.HOME.display()),
        CWD => println!("{}", shell.CWD()),
        PATH => {
            for (i, path) in shell.environment.PATH().iter().enumerate() {
                println!("[{i}]: {path}");
            }
        }
    }

    Ok(())
}

pub fn edit_path(shell: &mut ShellState, args: Vec<&str>) -> Result<()> {
    let arguments = clap_handle!(EditPathArgs::try_parse_from(args));
    use EditPathSubcommand::*;
    match arguments.subcommand {
        Append(AppendPathCommand { path }) => shell
            .environment
            .PATH_append(Path::try_from_path(&path, Some(&shell.environment.HOME))?),
        Prepend(PrependPathCommand { path }) => shell
            .environment
            .PATH_prepend(Path::try_from_path(&path, Some(&shell.environment.HOME))?),
        Insert(InsertPathCommand { index, path }) => shell.environment.PATH_insert(
            index,
            Path::try_from_path(&path, Some(&shell.environment.HOME))?,
        ),
        Delete(DeletePathCommand { index }) => shell.environment.PATH_delete(index),
    }
}
