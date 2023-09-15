use std::{path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand};

use crate::state::EnvVariable;

const TRUE_ARGS: [&str; 9] = [
    "true", "t", "enable", "enabled", "yes", "y", "on", "some", "1",
];
const FALSE_ARGS: [&str; 9] = [
    "false", "f", "disable", "disabled", "no", "n", "off", "none", "0",
];

#[derive(Parser, Debug)]
pub struct TestArgs {}

#[derive(Parser, Debug)]
pub struct ExitArgs {}

#[derive(Parser, Debug)]
pub struct WorkingDirectoryArgs {}

#[derive(Parser, Debug)]
pub struct ChangeDirectoryArgs {
    #[arg(help = "The path of the directory to switch to")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ListDirectoryArgs {
    #[arg(short = 'a', long = "all", help = "Show hidden files and directories")]
    pub show_hidden: bool,
    #[arg(short = 'l', long = "long", help = "Show files with details as a table")]
    pub long_view: bool, 
    #[arg(help = "The path of the directory to read")]
    pub path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct PreviousDirectoryArgs {}

#[derive(Parser, Debug)]
pub struct NextDirectoryArgs {}

#[derive(Parser, Debug)]
pub struct ClearTerminalArgs {}

#[derive(Parser, Debug)]
pub struct MakeFileArgs {
    #[arg(help = "The path of the file to create")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct MakeDirectoryArgs {
    #[arg(help = "The path of the directory to create")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct DeleteFileArgs {
    #[arg(help = "The path of the file to delete")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ReadFileArgs {
    #[arg(help = "The path of the file to read")]
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct RunExecutableArgs {
    #[arg(help = "The path to the executable")]
    pub path: PathBuf,
    #[arg(help = "The arguments to pass to the executable")]
    pub arguments: Vec<String>,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
pub struct ConfigureArgs {
    #[arg(
        long = "truncation",
        help = "The number of characters to trim each directory name to in the prompt"
    )]
    pub truncation: Option<MaybeUsize>,
    #[arg(
        long = "history-limit",
        help = "The maximum number of commands to store in the history"
    )]
    pub history_limit: Option<MaybeUsize>,
    #[arg(
        long = "multiline-prompt",
        help = "Whether to display the prompt on multiple lines"
    )]
    pub multiline_prompt: Option<Bool>,
    #[arg(long = "show-errors", help = "Whether to display error messages")]
    pub show_errors: Option<Bool>,
}

#[derive(Debug, Clone)]
pub enum Bool {
    True,
    False,
}

impl FromStr for Bool {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if TRUE_ARGS.contains(&s) {
            Ok(Bool::True)
        } else if FALSE_ARGS.contains(&s) {
            Ok(Bool::False)
        } else {
            Err("invalid boolean value".to_owned())
        }
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> Self {
        match b {
            Bool::True => true,
            Bool::False => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MaybeUsize {
    Some(usize),
    None,
}

impl FromStr for MaybeUsize {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = s.parse::<usize>() {
            Ok(MaybeUsize::Some(n))
        } else if FALSE_ARGS.contains(&s) {
            Ok(MaybeUsize::None)
        } else {
            Err("invalid integer or boolean value".to_owned())
        }
    }
}

impl From<MaybeUsize> for Option<usize> {
    fn from(n: MaybeUsize) -> Self {
        match n {
            MaybeUsize::Some(n) => Some(n),
            MaybeUsize::None => None,
        }
    }
}

#[derive(Parser, Debug)]
pub struct EnvironmentVariableArgs {
    #[arg(help = "The environment variable to display")]
    pub variable: EnvVariable,
}

#[derive(Parser, Debug)]
pub struct EditPathArgs {
    #[clap(subcommand)]
    pub subcommand: EditPathSubcommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EditPathSubcommand {
    #[clap(
        about = "Add the provided path to the end of the PATH variable so it is scanned last when resolving executables"
    )]
    Append(AppendPathCommand),
    #[clap(
        about = "Add the provided path to the beginning of the PATH variable so it is scanned first when resolving executables"
    )]
    Prepend(PrependPathCommand),
    #[clap(
        about = "Insert the provided path before the path at the specified index in the PATH variable"
    )]
    Insert(InsertPathCommand),
    #[clap(about = "Delete the path at the specified index in the PATH variable")]
    Delete(DeletePathCommand),
}

#[derive(Args, Debug, Clone)]
pub struct AppendPathCommand {
    #[arg(help = "The path to append to the PATH variable")]
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct PrependPathCommand {
    #[arg(help = "The path to prepend to the PATH variable")]
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct InsertPathCommand {
    #[arg(help = "The index to insert the provided path at")]
    pub index: usize,
    #[arg(help = "The path to insert into the PATH variable")]
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct DeletePathCommand {
    #[arg(help = "The index of the path to delete from the PATH variable")]
    pub index: usize,
}
