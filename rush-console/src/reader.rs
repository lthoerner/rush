use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};

use rush_state::context::Context;

// Represents a character that can be added to the line buffer, or an ENTER keypress, which will send the line buffer to the shell
// Keypresses that may have been handled downstream, but should not result in any further behavior, are represented by the Ignored variant
// Reprompt is a special case which will cause the prompt to be reprinted and the line buffer to be cleared, but will not return the buffer to the shell
// ? Maybe add a "clear linebuffer" switch to Reprompt
enum HandlerOutput {
    Char(char),
    Delete,
    Return,
    Reprompt,
    Ignored,
}

// Allows for reading a line of input from the user through the .read() method
// Handles all the actual terminal interaction between when the method is invoked and
// when the command is actually returned, such as line buffering etc
pub struct Console {
    stdout: Stdout,
}

impl Console {
    pub fn new() -> Self {
        Self { stdout: stdout() }
    }

    // TODO: Map crossterm errors to custom errors
    // Runs the REPL, returning as soon as a potential command is entered
    pub fn read(&mut self, context: &Context) -> Result<String> {
        let mut line_buffer = String::new();

        terminal::enable_raw_mode()?;
        self.print_prompt(context)?;
        let prompt_boundary = cursor::position()?.0;

        loop {
            execute!(self.stdout)?;
            let event = read()?;
            match self.handle_event(&event, prompt_boundary)? {
                HandlerOutput::Char(c) => line_buffer.push(c),
                HandlerOutput::Delete => {
                    line_buffer.pop();
                }
                HandlerOutput::Return => {
                    terminal::disable_raw_mode()?;
                    return Ok(line_buffer);
                }
                HandlerOutput::Reprompt => {
                    line_buffer.clear();
                    self.print_prompt(context)?;
                }
                HandlerOutput::Ignored => (),
            }
        }
    }

    // Handles a key event by queueing appropriate commands based on the given keypress
    // $ This is a temporary implementation for testing purposes only
    fn handle_event(&mut self, event: &Event, prompt_boundary: u16) -> Result<HandlerOutput> {
        let output;
        if let Event::Key(event) = event {
            match (event.modifiers, event.code) {
                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    queue!(self.stdout, Print(c))?;
                    output = HandlerOutput::Char(c)
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    if cursor::position()?.0 == prompt_boundary {
                        return Ok(HandlerOutput::Ignored);
                    }

                    self.backspace_char()?;
                    output = HandlerOutput::Delete
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    queue!(self.stdout, Print("\r\n"))?;
                    output = HandlerOutput::Return
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    queue!(self.stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
                    execute!(self.stdout)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    self.clear_terminal()?;
                    output = HandlerOutput::Reprompt
                }
                _ => output = HandlerOutput::Ignored,
            }
        } else {
            output = HandlerOutput::Ignored
        }

        // ? Error if not an Event::Key?
        Ok(output)
    }

    // Clears the entire terminal
    fn clear_terminal(&mut self) -> Result<()> {
        queue!(self.stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        Ok(())
    }

    // Queues the prompt to be printed
    fn print_prompt(&mut self, context: &Context) -> Result<()> {
        queue!(self.stdout, Print(generate_prompt(context)))?;
        Ok(())
    }

    // Queues a backspace or delete operation
    // TODO: Add a delete mode
    fn backspace_char(&mut self) -> Result<()> {
        queue!(
            self.stdout,
            cursor::MoveLeft(1),
            Print(' '),
            cursor::MoveLeft(1)
        )?;
        Ok(())
    }
}

// Generates the prompt string used by the REPL
fn generate_prompt(context: &Context) -> String {
    let user = context.env().USER().clone();
    let home = context.env().HOME();
    let truncation = context.shell_config().truncation_factor;
    let cwd = context.CWD().collapse(home, truncation);
    let prompt_delimiter = match context.shell_config().multi_line_prompt {
        true => "\r\n",
        false => " ",
    };

    // ? What is the actual name for this?
    let prompt_tick = match context.command_success {
        true => "❯".green(),
        false => "❯".red(),
    }
    .bold();

    format!(
        "\r\n{} on {}{}{} ",
        user.dark_blue(),
        cwd.dark_green(),
        prompt_delimiter,
        prompt_tick
    )
}
