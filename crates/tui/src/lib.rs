use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fluxion_core::{Action, Editor, Mode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
            Mode::SaveDialog => self.map_save_dialog_mode(key, editor),
            Mode::FilePicker => self.map_file_picker_mode(key),
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
            KeyCode::Char('[') => Action::PrevBuffer,
            KeyCode::Char(']') => Action::NextBuffer,
            KeyCode::Char('1') => Action::SwitchBuffer(1),
            KeyCode::Char('2') => Action::SwitchBuffer(2),
            KeyCode::Char('3') => Action::SwitchBuffer(3),
            KeyCode::Char('4') => Action::SwitchBuffer(4),
            KeyCode::Char('5') => Action::SwitchBuffer(5),
            KeyCode::Char('6') => Action::SwitchBuffer(6),
            KeyCode::Char('7') => Action::SwitchBuffer(7),
            KeyCode::Char('8') => Action::SwitchBuffer(8),
            KeyCode::Char('9') => Action::SwitchBuffer(9),
            KeyCode::Char('0') => Action::SwitchBuffer(0),
            KeyCode::Char(' ') => Action::EnterFilePicker,
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

    fn map_save_dialog_mode(&self, key: event::KeyEvent, editor: &mut Editor) -> Action {
        match key.code {
            KeyCode::Esc => Action::CancelDialog,
            KeyCode::Enter => {
                let filename = editor.command_input.clone();
                if !filename.is_empty() {
                    Action::SaveBufferAs(Some(std::path::PathBuf::from(filename)))
                } else {
                    Action::CancelDialog
                }
            }
            KeyCode::Backspace => Action::DeleteFromCommand,
            KeyCode::Char(c) => {
                editor.insert_into_command(c);
                Action::NoOp
            }
            _ => Action::NoOp,
        }
    }

    fn map_file_picker_mode(&self, key: event::KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => Action::FilePickerEsc,
            KeyCode::Enter => Action::FilePickerEnter,
            KeyCode::Char('j') => Action::FilePickerDown,
            KeyCode::Char('k') => Action::FilePickerUp,
            _ => Action::NoOp,
        }
    }

    fn render_ui(f: &mut ratatui::Frame, editor: &Editor) {
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(2),
                    Constraint::Length(3),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(f.area());

        let bufferline_area = vertical_chunks[0];
        let header_area = vertical_chunks[1];
        let status_area = vertical_chunks[2];
        let main_editor_area = vertical_chunks[3];

        Self::render_bufferline(f, editor, bufferline_area);
        Self::render_header(f, editor, header_area);
        Self::render_status(f, editor, status_area);
        Self::render_main_editor(f, editor, main_editor_area, status_area);

        if editor.mode == Mode::SaveDialog {
            Self::render_save_dialog(f, editor, f.area());
        }

        if editor.mode == Mode::FilePicker {
            Self::render_file_picker(f, editor, f.area());
        }
    }

    fn render_header(f: &mut ratatui::Frame, editor: &Editor, area: Rect) {
        let mode_text = match editor.mode {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Visual => "VISUAL",
            Mode::Command => "COMMAND",
            Mode::SaveDialog => "SAVE AS",
            Mode::FilePicker => "FILE PICKER",
        };

        let title = if editor.is_current_dirty() {
            format!("* {} - Fluxion", editor.get_current_title())
        } else {
            format!("{} - Fluxion", editor.get_current_title())
        };

        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" MODE: {} ", mode_text),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(title, Style::default().fg(Color::White)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Fluxion Editor")
                .border_style(Style::default().fg(Color::White)),
        );
        f.render_widget(header, area);
    }

    fn render_status(f: &mut ratatui::Frame, editor: &Editor, area: Rect) {
        let mode_help = match editor.mode {
            Mode::Normal => ":cmd i=ins v=vis ]/[/=prev/next Space+f=file",
            Mode::Insert => "Esc=normal",
            Mode::Visual => "Esc=normal",
            Mode::Command => "Enter=exec Esc=cancel",
            Mode::SaveDialog => "Enter=save Esc=cancel",
            Mode::FilePicker => "Enter=open j/k=navigate Esc=cancel",
        };

        let status_text = if editor.mode == Mode::Command {
            format!(":{}", editor.command_input)
        } else if editor.mode == Mode::SaveDialog {
            format!("Save as: {}", editor.command_input)
        } else {
            mode_help.to_string()
        };

        let status_area = Paragraph::new(status_text)
            .style(
                if editor.mode == Mode::Command || editor.mode == Mode::SaveDialog {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Yellow)
                },
            )
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_area, area);
    }

    fn render_bufferline(f: &mut ratatui::Frame, editor: &Editor, area: Rect) {
        let buffers = editor.get_buffers();
        let current_id = editor.buffer_manager.current_buffer_id();

        let mut buffer_spans: Vec<Span> = Vec::new();
        let mut current_width = 0;

        for buffer in buffers {
            let is_current = buffer.id == current_id;
            let dirty_mark = if buffer.dirty { " [+]" } else { "" };
            let buffer_text = format!(" {}:{}{}", buffer.id, buffer.title, dirty_mark);
            current_width += 2 + buffer_text.chars().count();

            let style = if is_current {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            buffer_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            buffer_spans.push(Span::styled(buffer_text, style));
        }

        buffer_spans.push(Span::styled(" |", Style::default().fg(Color::DarkGray)));

        let bufferline = Line::from(buffer_spans);

        let mut display_line = bufferline.clone();
        if current_width > area.width as usize {
            let truncated_text = bufferline
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect::<String>();
            let ellipsis = "...|";
            let available = area.width.saturating_sub(ellipsis.len() as u16) as usize;
            display_line = Line::from(format!(
                "{}{}",
                &truncated_text[..available.min(truncated_text.len())],
                ellipsis
            ));
        }

        let bufferline_widget =
            Paragraph::new(display_line).style(Style::default().fg(Color::Cyan));
        f.render_widget(bufferline_widget, area);
    }

    fn render_main_editor(f: &mut ratatui::Frame, editor: &Editor, area: Rect, status_area: Rect) {
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(6), Constraint::Min(0)].as_ref())
            .split(area);

        let line_numbers_area = horizontal_chunks[0];
        let text_area = horizontal_chunks[1];

        let text = editor.get_current_text();
        let max_lines = area.height.saturating_sub(2) as usize;
        let start_line = editor.scroll_offset;
        let end_line = (start_line + max_lines).min(text.len_lines());

        let line_number_style = Style::default().fg(Color::Gray);
        let mut line_number_lines: Vec<Line> = Vec::new();

        for i in start_line..end_line {
            let line_num = (i as i32 - editor.cursor.row as i32).abs();
            let style = if i == editor.cursor.row {
                line_number_style
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                line_number_style
            };
            line_number_lines.push(Line::from(vec![Span::styled(
                format!("{:>4}", line_num),
                style,
            )]));
        }

        let line_numbers = Paragraph::new(line_number_lines);
        f.render_widget(line_numbers, line_numbers_area);

        let mut text_lines: Vec<Line> = Vec::new();
        for i in start_line..end_line {
            let line = text.line(i);
            text_lines.push(Line::from(line.to_string()));
        }

        let paragraph = Paragraph::new(text_lines)
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Left);
        f.render_widget(paragraph, text_area);

        let cursor_row = editor.cursor.row.saturating_sub(editor.scroll_offset);
        let cursor_col = editor.cursor.col;

        let area_x = text_area.x + 1;
        let area_y = text_area.y + 1;

        if editor.mode == Mode::Command {
            let cursor_pos = editor.command_input.len() as u16 + 2;
            if cursor_pos + 2 < status_area.width {
                f.set_cursor_position((status_area.x + cursor_pos, status_area.y + 1));
            }
        } else if (editor.mode == Mode::Normal
            || editor.mode == Mode::Insert
            || editor.mode == Mode::Visual)
            && cursor_row < (text_area.height - 2) as usize
        {
            f.set_cursor_position((area_x + cursor_col as u16, area_y + cursor_row as u16));
        }
    }

    fn render_save_dialog(f: &mut ratatui::Frame, editor: &Editor, area: Rect) {
        let dialog_width = 50.min(area.width.saturating_sub(4));
        let dialog_height = 6;
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        let dialog_content = vec![
            Line::from("Save As"),
            Line::from(""),
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Green)),
                Span::styled(
                    editor.command_input.clone(),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from("Enter to save, Esc to cancel"),
        ];

        let dialog = Paragraph::new(dialog_content)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center);

        f.render_widget(Clear, dialog_area);
        f.render_widget(dialog, dialog_area);

        let cursor_pos = (editor.command_input.len() + 2) as u16;
        if cursor_pos < dialog_area.width - 2 {
            f.set_cursor_position((dialog_area.x + cursor_pos, dialog_area.y + 2));
        }
    }

    fn render_file_picker(f: &mut ratatui::Frame, editor: &Editor, area: Rect) {
        let picker = &editor.file_picker;

        let dialog_width = 60.min(area.width.saturating_sub(4));
        let dialog_height = 20.min(area.height.saturating_sub(4));
        let x = (area.width - dialog_width) / 2;
        let y = (area.height - dialog_height) / 2;

        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from("File Picker"));
        lines.push(Line::from(""));

        for (idx, file) in picker.files.iter().enumerate() {
            let icon = if file.is_dir { "📁 " } else { "📄 " };
            let style = if idx == picker.selected_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            lines.push(Line::from(vec![
                Span::styled(icon, Style::default()),
                Span::styled(&file.name, style),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Path: ", Style::default().fg(Color::Gray)),
            Span::styled(
                picker.current_dir.display().to_string(),
                Style::default().fg(Color::White),
            ),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from(
            "Enter: select/open | Esc: cancel | j/k: navigate",
        ));

        let dialog = Paragraph::new(lines)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Open File"));

        f.render_widget(Clear, dialog_area);
        f.render_widget(dialog, dialog_area);

        if picker.selected_idx < picker.files.len() {
            let cursor_y = dialog_area.y + 2 + picker.selected_idx as u16;
            if cursor_y < dialog_area.bottom() - 2 {
                f.set_cursor_position((dialog_area.x + 2, cursor_y));
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
