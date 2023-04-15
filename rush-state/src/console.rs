use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Alignment;
use ratatui::text::{Span, Spans, Text};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;

use crate::shell::Context;

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
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen, EnableMouseCapture)?;

        Ok(())
    }

    // Prompts the user for input
    pub fn prompt(&mut self, context: &Context) -> Result<()> {
        self.frame_buffer = generate_prompt(context);
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

// Appends the line buffer to the frame buffer so they can be rendered together but stored separately
// * This is used on frame updates where the line buffer is being edited
fn append_line<'a>(line: &'a str, buffer: &'a Text) -> Text<'a> {
    let mut temp_buffer = buffer.clone();
    
    if let Some(last_line) = temp_buffer.lines.last_mut() {
        last_line.0.push(Span::from(line));
    }

    temp_buffer
}
