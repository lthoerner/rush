// Represents a command that can be run by the prompt
pub struct Command {
    true_name: String,
    aliases: Vec<String>,
}

impl Command {
    pub fn new(true_name: &str, aliases: Vec<&str>) -> Self {
        let true_name = true_name.to_string();
        let aliases = aliases.iter().map(|a| a.to_string()).collect();

        Self {
            true_name,
            aliases,
        }
    }

    pub fn true_name(&self) -> &String {
        &self.true_name
    }
}

// Represents a collection of commands
// Allows for command resolution through aliases
pub struct CommandManager {
    commands: Vec<Command>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    // Adds a command to the manager
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
    }

    // Resolves a command name to a command
    // Returns None if the command is not found
    pub fn resolve(&self, name: &str) -> Option<&Command> {
        for command in &self.commands {
            if command.true_name == name {
                return Some(command)
            }

            for alias in &command.aliases {
                if alias == name {
                    return Some(command)
                }
            }
        }

        None
    }
}
