use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fluxion_core::{Action, Editor, Mode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
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

            if event::poll(std::time::Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
            {
                let action = self.map_key_to_action(key, editor);
                editor.handle_action(action);
            }
        }

        Ok(())
    }

    fn map_key_to_action(&self, key: event::KeyEvent, editor: &mut Editor) -> Action {
        match editor.mode {
            Mode::Normal => self.map_normal_mode(key),
            Mode::Insert => self.map_insert_mode(key),
            Mode::Visual => self.map_visual_mode(key),
            Mode::Command => self.map_command_mode(key, editor),
        }
    }

    fn map_normal_mode(&self, key: event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Char(':') => Action::EnterCommandMode,
            KeyCode::Char('h') => Action::MoveLeft,
            KeyCode::Char('j') => Action::MoveDown,
            KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Char('l') => Action::MoveRight,
            KeyCode::Char('i') => Action::EnterInsertMode,
            KeyCode::Char('v') => Action::EnterVisualMode,
            _ => Action::NoOp,
        }
    }

    fn map_insert_mode(&self, key: event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::EnterNormalMode,
            KeyCode::Enter => Action::Insert('\n'),
            KeyCode::Char(c) => Action::Insert(c),
            KeyCode::Backspace => Action::Delete,
            _ => Action::NoOp,
        }
    }

    fn map_visual_mode(&self, key: event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::EnterNormalMode,
            KeyCode::Char('h') => Action::MoveLeft,
            KeyCode::Char('j') => Action::MoveDown,
            KeyCode::Char('k') => Action::MoveUp,
            KeyCode::Char('l') => Action::MoveRight,
            _ => Action::NoOp,
        }
    }

    fn map_command_mode(&self, key: event::KeyEvent, editor: &mut Editor) -> Action {
        match key.code {
            KeyCode::Esc => Action::EnterNormalMode,
            KeyCode::Enter => Action::ExecuteCommand,
            KeyCode::Backspace => Action::DeleteFromCommand,
            KeyCode::Char(c) => {
                editor.insert_into_command(c);
                Action::NoOp
            }
            _ => Action::NoOp,
        }
    }

    fn render_ui(f: &mut ratatui::Frame, editor: &Editor) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(3),
                    Constraint::Percentage(92),
                ]
                .as_ref(),
            )
            .split(f.area());

        let mode_text = match editor.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
        };

        let header = Paragraph::new(format!("MODE: {}", mode_text))
            .style(Style::default().fg(Color::Green))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Fluxion Editor")
                    .border_style(Style::default().fg(Color::White)),
            );
        f.render_widget(header, vertical_chunks[0]);

        let mode_help = match editor.mode {
            Mode::Normal => "i=insert, v=visual, :=command",
            Mode::Insert => "Esc=normal mode",
            Mode::Visual => "Esc=normal mode",
            Mode::Command => "Esc=cancel, Enter=execute",
        };

        let status_text = if editor.mode == Mode::Command {
            format!(":{}", editor.command_input)
        } else {
            mode_help.to_string()
        };

        let status_area = Paragraph::new(status_text)
            .style(if editor.mode == Mode::Command {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Yellow)
            })
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_area, vertical_chunks[1]);

        let main_area = vertical_chunks[2];
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(8), Constraint::Min(0)].as_ref())
            .split(main_area);

        let line_numbers_area = horizontal_chunks[0];
        let text_area = horizontal_chunks[1];

        let max_lines = main_area.height.saturating_sub(2) as usize;
        let start_line = editor.scroll_offset;
        let end_line = (start_line + max_lines).min(editor.text.len_lines());

        let line_number_style = Style::default().fg(Color::Gray);
        let mut line_number_lines: Vec<Line> = Vec::new();

        for i in start_line..end_line {
            let relative_line_num = i.abs_diff(editor.cursor.row);
            line_number_lines.push(Line::from(vec![Span::styled(
                format!("{:>4}", relative_line_num),
                line_number_style,
            )]));
        }

        let line_numbers = Paragraph::new(line_number_lines)
            .style(Style::default().fg(Color::Gray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(line_numbers, line_numbers_area);

        let mut text_lines: Vec<Line> = Vec::new();
        for i in start_line..end_line {
            let line = editor.text.line(i);
            text_lines.push(Line::from(line.to_string()));
        }

        let paragraph = Paragraph::new(text_lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, text_area);

        let cursor_row = editor.cursor.row.saturating_sub(editor.scroll_offset);
        let cursor_col = editor.cursor.col;

        let area_x = text_area.x + 1;
        let area_y = text_area.y + 1;

        if (editor.mode == Mode::Normal
            || editor.mode == Mode::Insert
            || editor.mode == Mode::Visual)
            && cursor_row < (text_area.height - 2) as usize
        {
            f.set_cursor_position((area_x + cursor_col as u16, area_y + cursor_row as u16));
        }

        if editor.mode == Mode::Command {
            let cursor_pos = editor.command_input.len() as u16 + 1;
            if cursor_pos + 2 < vertical_chunks[1].width {
                f.set_cursor_position((
                    vertical_chunks[1].x + cursor_pos,
                    vertical_chunks[1].y + 1,
                ));
            }
        }
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
