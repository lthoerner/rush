use crate::errors::Result;
use crate::state::ShellState;

/// Represents either a builtin (internal command) or an executable (external command)
/// A `Runnable` may be executed by calling its `.run()` method
pub trait Runnable {
    fn run(&self, shell: &mut ShellState, arguments: Vec<&str>) -> Result<()>;
}

/// Wrapper type that makes it easier to read code related to builtins
pub struct Aliases {
    aliases: Vec<String>,
}

// * This implementation is here to make it easier to define aliases using string literals
impl From<Vec<&str>> for Aliases {
    fn from(aliases: Vec<&str>) -> Self {
        Self {
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
        }
    }
}

impl Aliases {
    pub fn contains(&self, alias: &str) -> bool {
        self.aliases.contains(&alias.to_string())
    }
}
