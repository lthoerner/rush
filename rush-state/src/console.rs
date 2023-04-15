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
    frame_buffer: Vec<Vec<Span<'a>>>,
}

impl<'a> Console<'a> {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            line_buffer: String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc. Sed euismod, nunc vel tincidunt lacinia, nunc nisl aliquam nisl, eu aliquam nisl nisl eu nunc."),
            frame_buffer: Vec::new(),
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
            let paragraph = Paragraph::new(convert_to_text(&self.frame_buffer, &self.line_buffer))
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
fn generate_prompt<'a>(context: &Context) -> Vec<Vec<Span<'a>>> {
    let mut result = Vec::new();
    let mut span_vec = Vec::new();

    let home = context.env().HOME();
    let truncation = context.shell_config().truncation_factor;
    let user = Span::styled(context.env().USER().clone(), Style::default().fg(Color::Blue));
    let cwd = Span::styled(context.env().CWD().collapse(home, truncation), Style::default().fg(Color::Green));

    span_vec.push(user);
    span_vec.push(Span::from(" on "));
    span_vec.push(cwd);

    // ? What is the actual name for this?
    let prompt_tick = Span::styled("❯ ", Style::default().add_modifier(Modifier::BOLD).fg(match context.success() {
        true => Color::LightGreen,
        false => Color::LightRed,
    }));

    // If the prompt is in multi-line mode, just append it to the first line and return
    // If the prompt is in single-line mode, create a new line and append it to the result, then return
    if context.shell_config().multi_line_prompt {
        result.push(span_vec);
        result.push(vec![prompt_tick]);
    } else {
        span_vec.push(Span::from(" "));
        span_vec.push(prompt_tick);
        result.push(span_vec);
    }

    result
}

// Converts a Vec<Vec<Span>> into a Text object which allows Console.draw() to create a frame
fn convert_to_text<'a>(text: &Vec<Vec<Span<'a>>>, line: &'a str) -> Text<'a> {
    let mut text = text.clone();

    // Append the line buffer to the last line of the frame buffer text
    let line = Span::from(line);
    text.last_mut().unwrap().push(line);

    // Convert the frame buffer text into a Vec<Spans> and then to a Text
    let spans: Vec<Spans> = text.iter().map(|s| Spans::from(s.clone())).collect();
    Text::from(spans)
}
