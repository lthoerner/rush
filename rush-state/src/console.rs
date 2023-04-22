use std::io::{stdout, Stdout};
use std::collections::HashSet;
use std::hash::Hash;

use anyhow::Result;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, DisableMouseCapture};
use crossterm::cursor;
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Layout, Direction, Constraint, Alignment};
use ratatui::text::{Span, Spans, Text};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Terminal, Frame};

use crate::shell::Shell;

// Represents an action that the handler instructs the REPL (Console.read()) to perform
// Allows for some actions to be performed in the handler and some to be performed in the REPL
enum ReplAction {
    // Instruction to return the line buffer to the shell and perform any necessary cleanup
    Return,
    // Instruction to exit the shell
    Exit,
    // Instruction to do nothing except update the TUI
    RedrawFrame,
    // Instruction to do nothing
    Ignore,
}

// More readable variant of a switch between "backspace" and "delete" keypresses for Console.remove_char()
#[derive(PartialEq)]
enum RemoveMode {
    Backspace,
    Delete,
}

// Represents a variety of switchable modes for clearing the TUI console/frame
// * Not to be confused with crossterm::terminal::ClearType
#[derive(PartialEq, Eq, Hash)]
enum ClearMode {
    // Whether to re-prompt the user after clearing the frame
    Prompt,
    // Whether to clear the line buffer
    ResetLineBuffer,
    // Whether to set the cursor index to the start of the line
    ResetCursor,
}

// Convenience struct for passing around a set of clear modes without duplicates
// ? Does this actually need to exist? Couldn't we just use [ClearMode; N]?
struct ClearModeBundle {
    modes: HashSet<ClearMode>,
}

impl<const N: usize> From<[ClearMode; N]> for ClearModeBundle {
    fn from(modes: [ClearMode; N]) -> Self {
        Self {
            modes: modes.into_iter().collect(),
        }
    }
}

impl ClearModeBundle {
    fn contains(&self, mode: ClearMode) -> bool {
        self.modes.contains(&mode)
    }
}

// Represents the TUI console
pub struct Console<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    line_buffer: String,
    frame_buffer: Text<'a>,
    // The index of the cursor in the line buffer
    // ? Should this be an Option<usize>?
    cursor_index: usize,
    scroll: usize,
}

impl<'a> Console<'a> {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            line_buffer: String::new(),
            frame_buffer: Text::default(),
            cursor_index: 0,
            scroll: 0,
        })
    }

    // Enters the TUI console
    pub fn enter(&mut self, shell: &Shell) -> Result<()> {
        enable_raw_mode()?;
        // ? Is mouse capture enabled by default?
        execute!(self.terminal.backend_mut(), EnterAlternateScreen, DisableMouseCapture)?;

        use ClearMode::*;
        self.clear(shell, [ResetLineBuffer, ResetCursor].into())
    }

    // Closes the TUI console
    pub fn close(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, cursor::MoveTo(0, 0), cursor::Show, Clear(ClearType::All))?;
        Ok(())
    }

    // Reads a line of input from the user
    // Handles all TUI interaction between the user and the prompt
    pub fn read_line(&mut self, shell: &Shell) -> Result<String> {
        // The line buffer must be reset manually because Console.prompt() does not clear it
        self.reset_line_buffer();
        self.prompt(shell)?;
        self.draw()?;

        loop {
            let event = event::read()?;
            let action = self.handle_event(event, shell)?;

            match action {
                ReplAction::Return => {
                    // Save the line buffer for returning and clear it to make way for the next Console.read_line() call
                    let line = self.line_buffer.clone();
                    self.line_buffer.clear();
                    
                    // Save the line buffer as part of the frame buffer
                    self.append_str(&line);
                    
                    return Ok(line)
                },
                ReplAction::Exit => {
                    self.close()?;
                    std::process::exit(0);
                },
                ReplAction::RedrawFrame => {
                    self.draw()?;
                },
                ReplAction::Ignore => (),
            }
        }
    }

    // Handles a key event by queueing appropriate commands based on the given keypress
    fn handle_event(&mut self, event: Event, shell: &Shell) -> Result<ReplAction> {
        use ClearMode::*;
        // TODO: Break up event handling into separate functions for different event categories
        match event {
            Event::Key(event) => {
                match (event.modifiers, event.code) {
                    (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => self.insert_char(c),
                    (KeyModifiers::NONE, KeyCode::Backspace) => self.remove_char(RemoveMode::Backspace),
                    (KeyModifiers::NONE, KeyCode::Delete) => self.remove_char(RemoveMode::Delete),
                    (KeyModifiers::NONE, KeyCode::Left) => self.move_cursor_left(),
                    (KeyModifiers::NONE, KeyCode::Right) => self.move_cursor_right(),
                    (KeyModifiers::NONE, KeyCode::Enter) if !self.line_buffer.is_empty() => return Ok(ReplAction::Return),
                    (KeyModifiers::NONE, KeyCode::Up) => self.scroll = self.scroll.saturating_sub(1),
                    (KeyModifiers::NONE, KeyCode::Down) => self.scroll = self.scroll.saturating_add(1),
                    // (KeyModifiers::NONE, KeyCode::Up) => self.scroll_history(HistoryDirection::Up, context)?,
                    // (KeyModifiers::NONE, KeyCode::Down) => self.scroll_history(HistoryDirection::Down, context)?,
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(ReplAction::Exit),
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => self.clear(shell, [Prompt].into())?,
                    _ => return Ok(ReplAction::Ignore),
                }
            }
            _ => return Ok(ReplAction::Ignore),
        }

        Ok(ReplAction::RedrawFrame)
    }

    // Appends a new prompt to the frame buffer, but does not perform a frame update,
    // and does not clear the line buffer or modify the cursor index
    fn prompt(&mut self, shell: &Shell) -> Result<()> {
        self.enforce_spacing();
        self.frame_buffer.extend(generate_prompt(shell));
        Ok(())
    }

    // Draws a TUI frame
    pub fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| Self::generate_frame(f, &self.line_buffer, &self.frame_buffer, self.scroll))?;
        Ok(())
    }

    // Generates a TUI frame based on the prompt/line buffer and frame buffer
    fn generate_frame(f: &mut Frame<CrosstermBackend<Stdout>>, line_buffer: &str, frame_buffer: &Text, scroll: usize) {
        // Create a Layout for the frame which reserves the bottom 20%
        // of the terminal for the prompt, and the rest for command output etc
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.size());

        let prompt_borders = Block::default().borders(Borders::ALL);
        let frame_borders = Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);
        
        // Create a Paragraph widget for the prompt
        let prompt_widget = Paragraph::new(line_buffer)
            .block(prompt_borders)
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        // Create a Paragraph widget for the frame buffer
        let frame_widget = Paragraph::new(frame_buffer.clone())
            .block(frame_borders)
            .style(Style::default())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        // Render the widgets
        f.render_widget(prompt_widget, chunks[1]);
        f.render_widget(frame_widget.scroll((scroll as u16, 0)), chunks[0]);
    }

    // Clears the screen and the line buffer and reprompts the user
    fn clear(&mut self, shell: &Shell, modes: ClearModeBundle) -> Result<()> {
        use ClearMode::*;

        // Clear the frame buffer
        self.frame_buffer = Text::default();

        if modes.contains(ResetLineBuffer) {
            // Resetting the line buffer requires the cursor index to also be reset,
            // regardless of whether the ResetCursor flag is provided or not
            self.reset_line_buffer();
        }

        if modes.contains(ResetCursor) {
            self.cursor_index = 0;
        }

        if modes.contains(Prompt) {
            self.prompt(shell)?;
        }

        Ok(())
    }

    // Inserts a character at the cursor position
    fn insert_char(&mut self, c: char) {
        self.line_buffer.insert(self.cursor_index, c);
        self.move_cursor_right();
    }

    // Removes a character from the line buffer at the cursor position
    fn remove_char(&mut self, mode: RemoveMode) {
        use RemoveMode::*;
        match mode {
            Backspace => {
                if self.cursor_index > 0 {
                    self.line_buffer.remove(self.cursor_index - 1);
                    self.move_cursor_left();
                }
            },
            Delete => {
                if self.cursor_index < self.line_buffer.len() {
                    self.line_buffer.remove(self.cursor_index);
                }
            },
        }
    }

    // Moves the cursor left by one character, checking for bounds
    fn move_cursor_left(&mut self) {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
        }
    }

    // Moves the cursor right by one character, checking for bounds
    fn move_cursor_right(&mut self) {
        if self.cursor_index < self.line_buffer.len() {
            self.cursor_index += 1;
        }
    }

    // Clears the line buffer and resets the cursor position
    fn reset_line_buffer(&mut self) {
        self.line_buffer.clear();
        self.cursor_index = 0;
    }

    // Prints a line of text to the console
    // TODO: Probably make this a macro in the future, but for now just make it use &str or String
    pub fn println(&mut self, text: &str) {
        self.append_newline(text);
        _ = self.draw()
    }

    // Appends a string to the frame buffer, splitting it into Spans by newline characters so it is rendered properly
    fn append_str(&mut self, string: &str) {
        // Return early on an empty string to allow for safely unwrapping the first line
        if string.is_empty() {
            return
        }

        // This code is awful so I will try to give my best description of it
        // First, we have to split the string into lines and convert them into Spans, because the Text type
        // does not render newline characters; instead, it requires that every line must be a separate Spans
        let mut spans = string.split('\n').map(str::to_owned).map(Spans::from);
        // To avoid automatically creating a new line before the text is printed (which would effectively forbid print!()-type behavior),
        // we have to append directly to the last Spans in the frame buffer
        // So this line basically grabs the Vec<Span> from the first Spans (first line)
        let first_spans = spans.next().unwrap().0;

        // If the frame buffer has any lines, we append the first line of the new text to the last line of the frame buffer
        // Otherwise, we just push the first line of the new text to the frame buffer in the form of a Spans,
        // so the first line of the new text isn't just skipped on an empty frame buffer
        if let Some(last_line) = self.frame_buffer.lines.last_mut() {
            last_line.0.extend(first_spans);
        } else {
            self.frame_buffer.lines.push(Spans::from(first_spans));
        }

        // The rest of the lines (Spans) can then be appended to the frame buffer as normal
        self.frame_buffer.extend(spans)
    }

    // Appends a string to the next line of the frame buffer
    fn append_newline(&mut self, string: &str) {
        self.append_str(&format!("\n{}", string))
    }

    // Ensures that there is an empty line at the end of the frame buffer
    // * This is used to make the prompt always appear one line below the last line of output, just for cosmetic purposes
    fn enforce_spacing(&mut self) {
        if let Some(last_line) = self.frame_buffer.lines.last_mut() {
            if !last_line.0.is_empty() {
                self.frame_buffer.lines.push(Spans::default());
            }
        }
    }
}

// Generates the prompt string used by the Console
// TODO: This will eventually need to not be hard-coded to allow for user customization
fn generate_prompt<'a>(shell: &Shell) -> Text<'a> {
    let mut span_list = Vec::new();

    let home = shell.env().HOME();
    let truncation = shell.config().truncation_factor;
    let user = Span::styled(shell.env().USER().clone(), Style::default().fg(Color::Blue));
    let cwd = Span::styled(shell.env().CWD().collapse(home, truncation), Style::default().fg(Color::Green));

    span_list.push(user);
    span_list.push(Span::from(" on "));
    span_list.push(cwd);

    // ? What is the actual name for this?
    let prompt_tick = Span::styled("❯ ", Style::default().add_modifier(Modifier::BOLD).fg(match shell.success() {
        true => Color::LightGreen,
        false => Color::LightRed,
    }));

    let mut spans = Vec::new();

    // If the prompt is in multi-line mode, create a new line and append it to the result, then return
    // If the prompt is in single-line mode, just append it to the first line and return
    if shell.config().multi_line_prompt {
        spans.push(Spans::from(span_list));
        spans.push(Spans::from(prompt_tick))
    } else {
        span_list.push(Span::from(" "));
        span_list.push(prompt_tick);
        spans.push(Spans::from(span_list));
    }

    Text::from(spans)
}

// Appends the line buffer to the frame buffer so they can be rendered together but stored separately
// * This is used on frame updates where the line buffer is being edited
fn append_line_buffer<'a>(line_buffer: &'a str, frame_buffer: &'a Text) -> Text<'a> {
    let mut temp_buffer = frame_buffer.clone();
    
    if let Some(last_line) = temp_buffer.lines.last_mut() {
        last_line.0.push(Span::from(line_buffer));
    }

    temp_buffer
}
