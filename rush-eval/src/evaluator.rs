use anyhow::Result;

use rush_state::context::Context;

use crate::commands::Dispatcher;

pub struct Evaluator {
    dispatcher: Dispatcher,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            dispatcher: Dispatcher::default(),
        }
    }

    // Evaluates and executes a command from a string
    pub fn eval(&self, context: &mut Context, command_name: String, command_args: Vec<String>) -> Result<()> {
        let command_name = command_name.as_str();
        let command_args = command_args.iter().map(|a| a.as_str()).collect();

        // Dispatch the command to the Dispatcher
        self.dispatcher.dispatch(command_name, command_args, context)
    }
}
