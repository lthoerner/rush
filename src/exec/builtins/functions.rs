/*
A quick write-up on Rush builtins:
Builtins are commands that are included with the shell. They are not able to be removed or modified without recompiling the shell.
Normally, a child process, such as a shell command, does not have direct access to the parent process's environment variables and other state.
However, the builtins are an exception to this rule. They are able to access the data because they are trusted to safely modify it.
Users are free to create their own builtins if they wish to modify the source code, but it comes with an inherent risk.

An executable will only have access to its arguments and environment variables, but not the shell's state, mostly for security reasons.
 */

use std::io::{stderr, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;

use clap::Parser;
use crossterm::cursor::MoveTo;
use crossterm::execute;
use crossterm::style::{Stylize, style};
use crossterm::terminal::{self, Clear, ClearType};
use file_owner::PathExt;
use chrono::offset::Local;
use chrono::DateTime;
use size::{Size, Style};


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

enum DirectoryListPermissionMode {
    Octal,
    String,
    Hidden,
}

enum DirectoryListTimestampMode {
    Modified,
    Created,
    Accessed,
    Hidden,
}

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
    let long_view = arguments.long_view;
    let octal_permissions = arguments.octal_permissions;
    let permission_seperator = arguments.permission_seperator;
    let use_modified_time = arguments.use_modified_time;
    let use_created_time = arguments.use_created_time;
    let use_accessed_time = arguments.use_accessed_time;
    let hide_timestamps = arguments.hide_timestamps;
    let hide_permissions = arguments.hide_permissions;
    let hide_user = arguments.hide_user;
    let hide_file_sizes = arguments.hide_file_sizes;
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
            directories.push(format!("{}/", fs_object_name).to_string());
        } else {
            files.push(fs_object_name.to_string());
        }
    }

    directories.sort();
    files.sort();

    if !long_view {
        for i in &directories {
            if i.starts_with('.') {
                print!("{}  ", i.clone().dark_green())
            } else {
                print!("{}  ", i.clone().green())
            }
        }

        for i in &files {
            if i.starts_with('.') {
                print!("{}  ", i.clone().grey())
            } else {
                print!("{}  ", i.clone().white())
            }

        }

        println!();

        return Ok(());
    }

    directories.append(&mut files);

    let permission_view = { 
        if hide_permissions {
            DirectoryListPermissionMode::Hidden
        } else {
            match octal_permissions {
                true => DirectoryListPermissionMode::Octal,
                false => DirectoryListPermissionMode::String,
            }
        }
    };

    let timestamp =  {
        if hide_timestamps {
            DirectoryListTimestampMode::Hidden
        } else {
            let mut seen_true = 0;
            let mut true_indx = 0;
            let times = [
                use_modified_time,
                use_accessed_time,
                use_created_time
            ];

            for (i, timemode) in times.iter().enumerate() {
                if *timemode {
                    seen_true += 1;
                    true_indx = i;
                }
            };

            match seen_true {
                1 => {
                    match true_indx {
                        0 => DirectoryListTimestampMode::Modified,
                        1 => DirectoryListTimestampMode::Accessed,
                        2 => DirectoryListTimestampMode::Created,
                        _ => DirectoryListTimestampMode::Modified,
                    }
                },
                _ => DirectoryListTimestampMode::Modified,
            }
        }
    };

    list_directory_long(directories, permission_view, path_to_read.clone(), permission_seperator, timestamp, hide_user, hide_file_sizes);

    Ok(())
}

fn list_directory_long(item: Vec<String>, permission_format: DirectoryListPermissionMode, cwd: PathBuf, permission_seperator: bool, timestamp: DirectoryListTimestampMode, hide_user: bool, hide_file_sizes: bool) {
    let path_to_read = cwd;
    let mut file_size_len: usize = 0;
    let mut file_size_len_last: usize = 0;
    let mut username_len: usize = 0;
    let mut username_len_last: usize = 0;
    
    if !hide_file_sizes {
        for i in &item {
            let path = path_to_read.join(i);
            let file_size = std::fs::metadata(path.to_path_buf()).unwrap().size();
            let formatted_fsize = Size::from_bytes(file_size).to_string();

            file_size_len_last = formatted_fsize.len();

            if file_size_len_last > file_size_len {
                    file_size_len = file_size_len_last;
            }
        }
    }

    if !hide_user {
        for i in &item {
            let path = path_to_read.join(i);
            
            username_len_last = path.owner().unwrap().to_string().len();

            if username_len_last > username_len {
                username_len = username_len_last;
            }
        }
    }

    for i in &item {
        let path = path_to_read.join(i);
        let permission_octal =  { 
            let x = format!("{:o}", std::fs::metadata(path.to_path_buf()).unwrap().permissions().mode()); 
            x[x.len() - 3..].to_string() 
        };

        let permissions = match permission_format {
            DirectoryListPermissionMode::Octal => {
                permission_octal.white()
            },
                
            DirectoryListPermissionMode::String => {
                let permission_str = permission_octal;
                let mut result = String::new();

                let dash = "-".dark_grey();
                let r = "r".white();
                let w = "w".white();
                let x = "x".white();

                let mapping = vec![
                    format!("{dash}{dash}{dash}"),
                    format!("{dash}{dash}{x}"), 
                    format!("{dash}{w}{dash}"), 
                    format!("{dash}{w}{x}"),
                    format!("{r}{dash}{dash}"),
                    format!("{r}{dash}{x}"),
                    format!("{r}{w}{dash}"),
                    format!("{r}{w}{x}")
                ];

                for (i, c) in permission_str.chars().enumerate() {
                    let digit = c.to_digit(8).unwrap() as usize;

                    if permission_seperator {
                        if i == 0 {
                            result.push_str(format!("{} ", "U".grey()).as_str());
                        }

                        if i == 1 {
                            result.push_str(format!(" {} ", "G".grey()).as_str());
                        }

                        if i == 2 {
                            result.push_str(format!(" {} ", "O".grey()).as_str());
                        }
                    }

                    result.push_str(&mapping[digit]);
                }

                result.white()
            },
            DirectoryListPermissionMode::Hidden => "".to_string().white(),
            
        };

        let file_size = std::fs::metadata(path.to_path_buf()).unwrap().size();
        let formatted_fsize = Size::from_bytes(file_size).to_string();
        let timestamp = match timestamp {
            DirectoryListTimestampMode::Modified => format!("{}", DateTime::<Local>::from(std::fs::metadata(path.to_path_buf()).unwrap().modified().unwrap()).format("%b %d %Y %T")).dark_cyan(),
            DirectoryListTimestampMode::Created => format!("{}", DateTime::<Local>::from(std::fs::metadata(path.to_path_buf()).unwrap().created().unwrap()).format("%b %d %Y %T")).dark_cyan(),
            DirectoryListTimestampMode::Accessed => format!("{}", DateTime::<Local>::from(std::fs::metadata(path.to_path_buf()).unwrap().accessed().unwrap()).format("%b %d %Y %T")).dark_cyan(),
            DirectoryListTimestampMode::Hidden => "".to_string().white(),
        };

        println!("{4}{3}{2}{1}{0}", 
            if i.starts_with('.') { 
                i.as_str().dark_grey() 
            } else if i.chars().last().unwrap() == '/' {
                if i.starts_with('.') {
                    i.as_str().dark_green()
                } else {
                    i.as_str().green() 
                }
            } else { 
                i.as_str().white() 
            }, 
            format!(" {timestamp} "),
            if !hide_user { format!(" {}{}", " ".repeat(username_len - path.owner().unwrap().to_string().len()), path.owner().unwrap().to_string().yellow()) } else { "".to_string() },
            if !hide_file_sizes { 
                if i.chars().last().unwrap() == '/' {
                    let spacing = " ".repeat(file_size_len - "-".len());
                    format!(" {}{}", spacing, "-".to_string().dark_grey()).white()
                } else { 
                    format!(" {}{}", formatted_fsize, " ".repeat(file_size_len - formatted_fsize.len())).green()
                }
            } else {
                "".to_string().white()
            },
            permissions
        );
    }
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
