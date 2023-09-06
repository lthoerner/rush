use crate::errors::Result;
use crate::exec::runnable::{Aliases, Runnable};
use crate::state::ShellState;

/// Represents a builtin function, its name and its aliases
pub struct Builtin {
    pub true_name: String,
    pub aliases: Aliases,
    #[allow(clippy::type_complexity)]
    function: Box<dyn Fn(&mut ShellState, Vec<&str>) -> Result<()>>,
}

impl Builtin {
    pub fn new<F: Fn(&mut ShellState, Vec<&str>) -> Result<()> + 'static>(
        true_name: &str,
        aliases: Vec<&str>,
        function: F,
    ) -> Self {
        let true_name = true_name.to_string();
        let aliases = Aliases::from(aliases);
        let function = Box::new(function);

        Self {
            true_name,
            aliases,
            function,
        }
    }
}

impl Runnable for Builtin {
    fn run(&self, shell: &mut ShellState, arguments: Vec<&str>) -> Result<()> {
        (self.function)(shell, arguments)
    }
}
