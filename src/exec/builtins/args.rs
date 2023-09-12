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
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ListDirectoryArgs {
    #[arg(short = 'a', long = "all")]
    pub show_hidden: bool,
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
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct MakeDirectoryArgs {
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct DeleteFileArgs {
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ReadFileArgs {
    pub path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct RunExecutableArgs {
    pub path: PathBuf,
    pub arguments: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct ConfigureArgs {
    #[arg(long = "truncation-factor")]
    pub truncation_factor: Option<MaybeUsize>,
    #[arg(long = "history-limit")]
    pub history_limit: Option<MaybeUsize>,
    #[arg(long = "multiline-prompt")]
    pub multiline_prompt: Option<FancyBool>,
    #[arg(long = "show-errors")]
    pub show_errors: Option<FancyBool>,
}

#[derive(Debug, Clone)]
pub enum FancyBool {
    True,
    False,
}

impl FromStr for FancyBool {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if TRUE_ARGS.contains(&s) {
            Ok(FancyBool::True)
        } else if FALSE_ARGS.contains(&s) {
            Ok(FancyBool::False)
        } else {
            Err("invalid boolean value".to_owned())
        }
    }
}

impl From<FancyBool> for bool {
    fn from(b: FancyBool) -> Self {
        match b {
            FancyBool::True => true,
            FancyBool::False => false,
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
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct PrependPathCommand {
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct InsertPathCommand {
    pub index: usize,
    pub path: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct DeletePathCommand {
    pub index: usize,
}
