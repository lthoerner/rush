use clap::Parser;

#[derive(Parser, Debug)]
#[command(no_binary_name = true)]
pub struct ListDirectoryArguments {
    #[clap(short, long, default_value_t = false)]
    pub all: bool,
    pub path: Option<String>,
}