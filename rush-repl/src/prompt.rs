#![allow(dead_code, unused_imports, unused_variables, unreachable_code)]

use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::cursor;
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue, ExecutableCommand};

use rush_shell::commands::Context;

// Represents a character that can be added to the line buffer, or an ENTER keypress, which will send the line buffer to the shell
// Keypresses that may have been handled downstream, but should not result in any further behavior, are represented by the Ignored variant
// ? EventOutput or HandlerOutput?
enum EventOutput {
    Char(char),
    Delete,
    Return,
    Ignored,
}

// ? What should this be named?
// ? Does it need to be a struct?
pub struct Repl {
    stdout: Stdout,
}

impl Repl {
    pub fn new() -> Self {
        Self { stdout: stdout() }
    }

    // TODO: Map crossterm errors to custom errors
    // Runs the REPL, returning as soon as a potential command is entered
    // Handles all the actual terminal interaction between when the method is invoked and
    // when the command is actually returned, such as line buffering etc.
    pub fn read(&mut self, context: &Context) -> Result<String> {
        let mut line_buffer = String::new();

        terminal::enable_raw_mode()?;
        print_prompt(&mut self.stdout, context)?;
        let prompt_boundary = cursor::position()?.0;

        loop {
            execute!(self.stdout)?;
            let event = read()?;
            match self.handle_event(&event, prompt_boundary, context)? {
                EventOutput::Char(c) => line_buffer.push(c),
                EventOutput::Delete => { line_buffer.pop(); }
                EventOutput::Return => {
                    terminal::disable_raw_mode()?;
                    return Ok(line_buffer)
                }
                EventOutput::Ignored => (),
            }
        }
    }

    // Handles a key event by queueing appropriate commands based on the given keypress
    // $ This is a temporary implementation for testing purposes only
    fn handle_event(&mut self, event: &Event, prompt_boundary: u16, context: &Context) -> Result<EventOutput> {
        let output;
        if let Event::Key(event) = event {
            match (event.modifiers, event.code) {
                (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                    queue!(self.stdout, Print(c))?;
                    output = EventOutput::Char(c)
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    if cursor::position()?.0 == prompt_boundary {
                        return Ok(EventOutput::Ignored)
                    }

                    backspace_char(&mut self.stdout)?;
                    output = EventOutput::Delete
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    queue!(self.stdout, Print("\r\n"))?;
                    output = EventOutput::Return
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                    queue!(self.stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
                    execute!(self.stdout)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                _ => output = EventOutput::Ignored,
            }
        } else {
            output = EventOutput::Ignored
        }

        // ? Error if not an Event::Key?
        Ok(output)
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
    let prompt_tick = "❯";

    format!("\r\n{} on {}{}{} ", user.dark_blue(), cwd.dark_green(), prompt_delimiter, prompt_tick.green().bold())
}

// Clears the entire terminal
fn clear_terminal(stdout: &mut Stdout) -> Result<()> {
    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    execute!(stdout)?;
    Ok(())
}

// Queues the prompt to be printed
fn print_prompt(stdout: &mut Stdout, context: &Context) -> Result<()> {
    queue!(stdout, Print(generate_prompt(context)))?;
    Ok(())
}

// Queues a backspace or delete operation
// TODO: Add a delete mode
fn backspace_char(stdout: &mut Stdout) -> Result<()> {
    queue!(stdout, cursor::MoveLeft(1), Print(' '), cursor::MoveLeft(1))?;
    Ok(())
}
