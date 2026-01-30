use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fluxion_core::{Action, Editor};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::{error::Error, io};

/// Handles the Terminal User Interface
pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn run(&mut self, editor: &mut Editor) -> Result<(), Box<dyn Error>> {
        while !editor.should_quit {
            self.terminal.draw(|f| {
                Self::render_ui(f, editor);
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    let action = self.map_key_to_action(key);
                    editor.handle_action(action);
                }
            }
        }

        Ok(())
    }

    fn map_key_to_action(&self, key: event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::Quit,
            KeyCode::Char(c) => Action::Insert(c),
            KeyCode::Backspace => Action::Delete,
            _ => Action::NoOp,
        }
    }

    fn render_ui(f: &mut ratatui::Frame, editor: &Editor) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(94),
                    Constraint::Percentage(1),
                ]
                .as_ref(),
            )
            .split(f.area());

        let block = Block::default().title("Block").borders(Borders::ALL);
        f.render_widget(block, chunks[0]);

        let area = chunks[1];
        let max_lines = area.height as usize;
        let mut visible_text = String::new();

        let start_line = editor.scroll_offset;
        let end_line = (start_line + max_lines).min(editor.text.len_lines());

        for i in start_line..end_line {
            let line = editor.text.line(i);
            visible_text.push_str(&line.to_string());
        }

        let paragraph = Paragraph::new(visible_text)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Fluxion Editor")
                    .border_style(Style::default().fg(Color::White)),
            );
        f.render_widget(paragraph, chunks[1]);

        let footer = Paragraph::new("Press 'Esc' to quit")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
