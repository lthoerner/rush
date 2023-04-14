use std::io::{stdout, Stdout};
use std::ops::Range;

use anyhow::Result;
use crossterm::cursor::{self, MoveToNextLine};
use crossterm::event::{read, Event, KeyCode, KeyModifiers};
use crossterm::style::{Print, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};

use rush_state::shell::Context;

// Represents an action that the handler instructs the REPL (Console.read()) to perform
// Allows for some actions to be performed in the handler and some to be performed in the REPL
enum ReplAction {
    // Instruction to return the line buffer to the shell and perform any necessary cleanup
    Return,
    // Instruction to clear the line buffer and re-prompt the user
    Clear,
    // Instruction to exit the shell
    Exit,
    // Instruction to do nothing
    Ignore,
}

// More readable variant of a switch between "backspace" and "delete" keypresses for Console.remove_char()
#[derive(PartialEq)]
enum RemoveMode {
    Backspace,
    Delete,
}

// Represents either a "history up" or "history down" keypress (arrow keys)
#[derive(PartialEq)]
enum HistoryDirection {
    Up,
    Down,
}

// Allows for reading a line of input from the user through the .read() method
// Handles all the actual terminal interaction between when the method is invoked and
// when the command is actually returned, such as line buffering etc
pub struct Console {
    // * Stdout is stored to prevent repeated std::io::stdout() calls
    stdout: Stdout,
    // A string that stores the current line of input
    // When the user hits ENTER, the line buffer is returned to the shell
    line_buffer: String,
    // If the user is scrolling through the command history, this stores the original line buffer so it can be restored if needed
    history_buffer: Option<String>,
    // The history index stored when the user is scrolling through the command history
    history_index: Option<usize>,
    // The "coordinate" of the cursor is a one-dimensional index of the cursor in the buffer
    cursor_coord: usize,
    // The X-offset of the start of the user input from the prompt
    prompt_offset: u16,
}

impl Console {
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            line_buffer: String::new(),
            history_buffer: None,
            history_index: None,
            cursor_coord: 0,
            // ? Should this be an Option? It would only be None in the constructor but it might make more sense than defaulting to 0...
            prompt_offset: 0,
        }
    }

    // TODO: Map crossterm errors to custom errors
    // Prompts the user and handles all input keypresses/resulting terminal interaction up until a line of input is entered
    pub fn read(&mut self, context: &Context) -> Result<String> {
        terminal::enable_raw_mode()?;
        self.print_prompt(context)?;
        // The prompt offset is calculated on the first time the user is prompted by simply checking the cursor's X position
        // ? Is there a better way to determine this?
        self.prompt_offset = cursor::position()?.0;

        loop {
            execute!(self.stdout)?;
            let event = read()?;
            let action = self.handle_event(event, context)?;

            // self.print_debug_text(1, format!("Raw buffer: {}", self.line_buffer))?;
            // self.print_debug_text(1, format!("Terminal X size: {} | Terminal Y size: {}", terminal::size()?.0, terminal::size()?.1))?;
            // self.print_debug_text(2, format!("Cursor X: {} | Cursor Y: {}", cursor::position()?.0, cursor::position()?.1))?;

            match action {
                ReplAction::Return => {
                    execute!(self.stdout, MoveToNextLine(1))?;
                    terminal::disable_raw_mode()?;
                    let line = self.line_buffer.clone();
                    self.line_buffer.clear();
                    self.cursor_coord = 0;
                    self.clear_debug_text(1..2)?;
                    return Ok(line);
                }
                ReplAction::Clear => {
                    self.line_buffer.clear();
                    self.cursor_coord = 0;
                    self.clear_terminal()?;
                    self.print_prompt(context)?;
                }
                ReplAction::Exit => {
                    self.clear_terminal()?;
                    execute!(self.stdout)?;
                    terminal::disable_raw_mode()?;
                    std::process::exit(0);
                }
                ReplAction::Ignore => (),
            }
        }
    }

    // Handles a key event by queueing appropriate commands based on the given keypress
    fn handle_event(&mut self, event: Event, context: &Context) -> Result<ReplAction> {
        // TODO: Break up event handling into separate functions for different event categories
        match event {
            Event::Key(event) => {
                match (event.modifiers, event.code) {
                    (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => self.insert_char(c)?,
                    (KeyModifiers::NONE, KeyCode::Backspace) => self.remove_char(RemoveMode::Backspace)?,
                    (KeyModifiers::NONE, KeyCode::Delete) => self.remove_char(RemoveMode::Delete)?,
                    (KeyModifiers::NONE, KeyCode::Left) => self.move_cursor_left()?,
                    (KeyModifiers::NONE, KeyCode::Right) => self.move_cursor_right()?,
                    (KeyModifiers::NONE, KeyCode::Enter) if !self.line_buffer.is_empty() => return Ok(ReplAction::Return),
                    (KeyModifiers::NONE, KeyCode::Up) => self.scroll_history(HistoryDirection::Up, context)?,
                    (KeyModifiers::NONE, KeyCode::Down) => self.scroll_history(HistoryDirection::Down, context)?,
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(ReplAction::Exit),
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => return Ok(ReplAction::Clear),
                    _ => (),
                }
            }
            // Event::Resize(x, y) => {
            //     self.clear_terminal()?;
            //     self.print_debug_text(1, format!("X-size: {x} | Y-size: {y}"))?
            // }
            _ => (),
        }

        Ok(ReplAction::Ignore)
    }

    // Moves the cursor to the right in both the terminal and the line buffer, provided the cursor is not at the end of the line
    fn move_cursor_right(&mut self) -> Result<()> {
        if self.cursor_coord != self.line_buffer.len() {
            self.move_cursor_terminal_right()?;
            self.cursor_coord += 1;
        }

        Ok(())
    }

    // Moves the cursor to the left in both the terminal and the line buffer, provided the cursor is not at the start of the line
    fn move_cursor_left(&mut self) -> Result<()> {
        if self.cursor_coord != 0 {
            self.move_cursor_terminal_left()?;
            self.cursor_coord -= 1;
        }

        Ok(())
    }

    // Moves the cursor to the right in the terminal, wrapping to the next line if necessary
    fn move_cursor_terminal_right(&mut self) -> Result<()> {
        let x_size = terminal::size()?.0;
        let x_pos = cursor::position()?.0;

        if x_pos == x_size - 1 {
            queue!(self.stdout, cursor::MoveToNextLine(1))?;
        } else {
            queue!(self.stdout, cursor::MoveRight(1))?;
        }

        Ok(())
    }

    // Moves the cursor to the left in the terminal, wrapping to the previous line if necessary
    fn move_cursor_terminal_left(&mut self) -> Result<()> {
        let x_size = terminal::size()?.0;
        let x_pos = cursor::position()?.0;

        if x_pos == 0 {
            queue!(
                self.stdout,
                cursor::MoveToPreviousLine(1),
                cursor::MoveRight(x_size - 1)
            )?;
        } else {
            queue!(self.stdout, cursor::MoveLeft(1))?;
        }

        Ok(())
    }

    // Inserts a character into the line buffer at the cursor position
    fn insert_char(&mut self, char: char) -> Result<()> {
        // Insert the char and update the buffer after the cursor
        self.line_buffer.insert(self.cursor_coord, char);
        self.print_buffer_section(false)?;
        self.cursor_coord += 1;
        // Move the cursor right so the text does not get overwritten upon the next insertion
        self.move_cursor_terminal_right()?;

        Ok(())
    }

    // Removes the character either immediately preceding the cursor position or the character at
    // the cursor position, depending on whether in Backspace or Delete mode, respectively
    fn remove_char(&mut self, mode: RemoveMode) -> Result<()> {
        use RemoveMode::*;
        match mode {
            Backspace => {
                if self.cursor_coord == 0 {
                    return Ok(())
                } else {
                    self.cursor_coord -= 1;
                }
            }
            Delete => {
                if self.cursor_coord == self.line_buffer.len() {
                    return Ok(())
                }
            }
        }

        self.line_buffer.remove(self.cursor_coord);

        if mode == Backspace {
            self.move_cursor_terminal_left()?;
        }

        self.print_buffer_section(true)?;

        Ok(())
    }

    // Prints a section of the line buffer starting from the cursor position
    fn print_buffer_section(&mut self, deletion_mode: bool) -> Result<()> {
        // If deleting a character, print a space at the end of the buffer to prevent
        // the last char in the buffer from being duplicated when shifting the line
        // * This is a better solution than first clearing the line after the cursor
        // * because clearing the line incurs a more noticeable flicker
        let deletion_char = match deletion_mode {
            true => " ",
            false => "",
        };

        queue!(
            self.stdout,
            cursor::SavePosition,
            Print(&self.line_buffer[self.cursor_coord..]),
            Print(deletion_char),
            cursor::RestorePosition,
        )?;

        Ok(())
    }

    // Prints debug text to the bottom lines of the terminal
    #[allow(dead_code)]
    fn print_debug_text(&mut self, line: u16, text: String) -> Result<()> {
        queue!(
            self.stdout,
            cursor::SavePosition,
            cursor::MoveTo(0, terminal::size()?.1 - line),
            Clear(ClearType::UntilNewLine),
            Print(text),
            cursor::RestorePosition,
        )?;

        Ok(())
    }

    // Clears the bottom lines of the terminal
    fn clear_debug_text(&mut self, lines: Range<u16>) -> Result<()> {
        for line in lines {
            queue!(
                self.stdout,
                cursor::SavePosition,
                cursor::MoveTo(0, terminal::size()?.1 - line),
                Clear(ClearType::UntilNewLine),
                cursor::RestorePosition,
            )?;
        }

        Ok(())
    }

    // // Reprints all text in stdout after the terminal is resized
    // fn resize_terminal(&mut self) -> Result<()> {
    //     // let stdout = self.stdout
    //     // self.clear_terminal()?;
    //     // Ok(())
    //     todo!()
    // }

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

    // Reprints the entire line buffer and moves the cursor to the end
    fn reset_line(&mut self) -> Result<()> {
        // $ This will definitely cause a bug when the buffer is multiple lines long
        queue!(
            self.stdout,
            cursor::MoveToColumn(self.prompt_offset),
            Clear(ClearType::UntilNewLine),
            Print(&self.line_buffer)
        )?;
        self.cursor_coord = self.line_buffer.len();
        Ok(())
    }

    // Scrolls through the Shell's command history
    fn scroll_history(&mut self, direction: HistoryDirection, context: &Context) -> Result<()> {
        use HistoryDirection::*;
        let history = context.history();
        let history_len = history.len();
        let history_last_index = history_len - 1;

        // If the history is empty, do nothing
        if history.is_empty() {
            return Ok(());
        }

        match self.history_index {
            // If the user is already scrolling through the history, move the index in the appropriate direction
            // If they attempt to scroll past the end of the history, restore the original line buffer
            Some(index) => {
                match direction {
                    Up => {
                        // Prevent the user from scrolling out of bounds
                        if index == 0 {
                            return Ok(());
                        } else {
                            self.history_index = Some(index - 1)
                        }
                    }
                    Down => {
                        // If the user scrolls back past the start of the history, restore the original line buffer
                        if index == history_last_index {
                            // TODO: Change this to an actual error
                            self.line_buffer = self
                                .history_buffer
                                .clone()
                                .expect("History buffer was not found when it should exist");
                            self.history_buffer = None;
                            self.history_index = None;
                        } else {
                            self.history_index = Some(index + 1)
                        }
                    }
                }
            }
            // If the user is just starting to scroll through the history, start at the most recent history
            // If they attempt to scroll past the end of the history, do nothing
            None => {
                match direction {
                    Up => {
                        // * Bounds check is not needed in this case because it is guaranteed that history
                        // * contains at least one element due to the .is_empty() check
                        self.history_index = Some(history_last_index);
                        self.history_buffer = Some(self.line_buffer.clone());
                    }
                    Down => return Ok(()),
                }
            }
        }

        // TODO: Change this to an actual error
        if let Some(index) = self.history_index {
            self.line_buffer = history
                .get(index)
                .expect("Tried to access non-existent command history")
                .clone();
        }

        self.reset_line()
    }
}

// Generates the prompt string used by the REPL
fn generate_prompt(context: &Context) -> String {
    let user = context.env().USER().clone();
    let home = context.env().HOME();
    let truncation = context.shell_config().truncation_factor;
    let cwd = context.env().CWD().collapse(home, truncation);
    let prompt_delimiter = match context.shell_config().multi_line_prompt {
        true => "\r\n",
        false => " ",
    };

    // ? What is the actual name for this?
    let prompt_tick = match context.success() {
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
