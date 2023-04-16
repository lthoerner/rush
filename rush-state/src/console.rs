use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, DisableMouseCapture, EnableMouseCapture};
use crossterm::cursor;
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::text::{Span, Spans, Text};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;

use crate::shell::Context;

// Represents an action that the handler instructs the REPL (Console.read()) to perform
// Allows for some actions to be performed in the handler and some to be performed in the REPL
enum ReplAction {
    // Instruction to return the line buffer to the shell and perform any necessary cleanup
    Return,
    // Instruction to clear the line buffer and re-prompt the user
    Clear,
    // Instruction to exit the shell
    Exit,
    // Instruction to do nothing except update the TUI
    RedrawFrame,
    // Instruction to do nothing
    Ignore,
}

// Represents the TUI console
pub struct Console<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    line_buffer: String,
    frame_buffer: Text<'a>,
}

impl<'a> Console<'a> {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            line_buffer: String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc."),
            frame_buffer: Text::default(),
        })
    }

    // Enters the TUI console
    pub fn enter(&mut self) -> Result<()> {
        self.terminal.clear()?;
        enable_raw_mode()?;
        // ? Is mouse capture enabled by default?
        execute!(self.terminal.backend_mut(), EnterAlternateScreen, DisableMouseCapture)?;

        Ok(())
    }

    // Closes the TUI console
    pub fn close(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, EnableMouseCapture, cursor::MoveTo(0, 0), cursor::Show, Clear(ClearType::All))?;

        Ok(())
    }

    // Reads a line of input from the user
    // Handles all TUI interaction between the user and the prompt
    pub fn read_line(&mut self, context: &Context) -> Result<String> {
        self.prompt(context)?;
        self.draw()?;

        loop {
            let event = event::read()?;
            let action = self.handle_event(event)?;

            match action {
                ReplAction::Return => {
                    // Save the line buffer for returning and clear it to make way for the next Console.read_line() call
                    let line = self.line_buffer.clone();
                    self.line_buffer.clear();
                    
                    // Save the line buffer as part of the frame buffer
                    append_str(&line, &mut self.frame_buffer);
                    
                    return Ok(line)
                },
                ReplAction::Clear => {
                    self.terminal.clear()?;
                    self.line_buffer.clear();
                    self.prompt(context)?;
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
    fn handle_event(&mut self, event: Event) -> Result<ReplAction> {
        // TODO: Break up event handling into separate functions for different event categories
        match event {
            Event::Key(event) => {
                match (event.modifiers, event.code) {
                    (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                        self.line_buffer.push(c);
                        return Ok(ReplAction::RedrawFrame);
                    },
                    // * This has to be surrounded by brackets in order to ignore the return value of String.pop()
                    (KeyModifiers::NONE, KeyCode::Backspace) => {
                        self.line_buffer.pop();
                        return Ok(ReplAction::RedrawFrame);
                    },
                    // (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => self.insert_char(c)?,
                    // (KeyModifiers::NONE, KeyCode::Backspace) => self.remove_char(RemoveMode::Backspace)?,
                    // (KeyModifiers::NONE, KeyCode::Delete) => self.remove_char(RemoveMode::Delete)?,
                    // (KeyModifiers::NONE, KeyCode::Left) => self.move_cursor_left()?,
                    // (KeyModifiers::NONE, KeyCode::Right) => self.move_cursor_right()?,
                    (KeyModifiers::NONE, KeyCode::Enter) if !self.line_buffer.is_empty() => return Ok(ReplAction::Return),
                    // (KeyModifiers::NONE, KeyCode::Up) => self.scroll_history(HistoryDirection::Up, context)?,
                    // (KeyModifiers::NONE, KeyCode::Down) => self.scroll_history(HistoryDirection::Down, context)?,
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Ok(ReplAction::Exit),
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => return Ok(ReplAction::Clear),
                    _ => (),
                }
            }
            Event::Resize(_, _) => return Ok(ReplAction::RedrawFrame),
            _ => (),
        }

        Ok(ReplAction::Ignore)
    }

    // Prompts the user for input
    fn prompt(&mut self, context: &Context) -> Result<()> {
        append_newline(&mut self.frame_buffer);
        self.frame_buffer.extend(generate_prompt(context));
        self.draw()
    }

    // Draws a TUI frame
    fn draw(&mut self) -> Result<()> {
        self.terminal.draw(|f| {
            let size = f.size();
            let paragraph = Paragraph::new(append_line(&self.line_buffer, &mut self.frame_buffer))
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default())
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, size);
        })?;
        
        Ok(())
    }
}

// Generates the prompt string used by the Console
fn generate_prompt<'a>(context: &Context) -> Text<'a> {
    let mut span_list = Vec::new();

    let home = context.env().HOME();
    let truncation = context.shell_config().truncation_factor;
    let user = Span::styled(context.env().USER().clone(), Style::default().fg(Color::Blue));
    let cwd = Span::styled(context.env().CWD().collapse(home, truncation), Style::default().fg(Color::Green));

    span_list.push(user);
    span_list.push(Span::from(" on "));
    span_list.push(cwd);

    // ? What is the actual name for this?
    let prompt_tick = Span::styled("❯ ", Style::default().add_modifier(Modifier::BOLD).fg(match context.success() {
        true => Color::LightGreen,
        false => Color::LightRed,
    }));

    let mut spans = Vec::new();

    // If the prompt is in multi-line mode, create a new line and append it to the result, then return
    // If the prompt is in single-line mode, just append it to the first line and return
    if context.shell_config().multi_line_prompt {
        spans.push(Spans::from(span_list));
        spans.push(Spans::from(prompt_tick))
    } else {
        span_list.push(Span::from(" "));
        span_list.push(prompt_tick);
        spans.push(Spans::from(span_list));
    }

    Text::from(spans)
}

// Appends a string to the frame buffer
fn append_str<'a, 'b>(string: &'a str, buffer: &mut Text<'b>) {
    // The string must be appended to the last Spans object in the Text object,
    // because otherwise it would be rendered on a new line
    if let Some(last_line) = buffer.lines.last_mut() {
        last_line.0.push(Span::from(string.to_string()));
    }
}

// Adds a line break to the end of the framebuffer if the last line is not empty
// This effectively makes sure that the prompt is always rendered one line below the last line
fn append_newline<'a>(buffer: &mut Text<'a>) {
    if let Some(last_line) = buffer.lines.last_mut() {
        if !last_line.0.is_empty() {
            buffer.extend(Text::from("\n"));
        }
    }
}

// Appends the line buffer to the frame buffer so they can be rendered together but stored separately
// * This is used on frame updates where the line buffer is being edited
fn append_line<'a>(line: &'a str, buffer: &'a Text) -> Text<'a> {
    let mut temp_buffer = buffer.clone();
    
    if let Some(last_line) = temp_buffer.lines.last_mut() {
        last_line.0.push(Span::from(line));
    }

    temp_buffer
}
