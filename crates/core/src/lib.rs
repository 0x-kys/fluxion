use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

pub enum Action {
    Quit,
    Insert(char),
    Delete,
    DeleteFromCommand,
    NoOp,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterInsertMode,
    EnterNormalMode,
    EnterVisualMode,
    EnterCommandMode,
    ExecuteCommand,
}

/// The core editor state.
pub struct Editor {
    pub text: Rope,
    pub cursor: Cursor,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub mode: Mode,
    pub command_input: String,
}

impl Editor {
    pub fn new(initial_text: &str) -> Self {
        Self {
            text: Rope::from_str(initial_text),
            cursor: Cursor::new(0, 0),
            scroll_offset: 0,
            should_quit: false,
            mode: Mode::Normal,
            command_input: String::new(),
        }
    }

    fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.clamp_col_to_line();
        }
    }

    fn move_down(&mut self) {
        if self.cursor.row < self.text.len_lines().saturating_sub(1) {
            self.cursor.row += 1;
            self.clamp_col_to_line();
        }
    }

    fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = self.text.line(self.cursor.row).len_chars();
        }
    }

    fn move_right(&mut self) {
        let line_len = self.text.line(self.cursor.row).len_chars();
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.row < self.text.len_lines().saturating_sub(1) {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    fn clamp_col_to_line(&mut self) {
        let line_len = self.text.line(self.cursor.row).len_chars();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    fn insert_char(&mut self, c: char) {
        let byte_pos = self.cursor_to_byte();
        self.text.insert_char(byte_pos, c);
        if c == '\n' {
            self.cursor.row += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
    }

    fn delete_char(&mut self) {
        let byte_pos = self.cursor_to_byte();
        if byte_pos > 0 {
            self.text.remove(byte_pos - 1..byte_pos);
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
            } else if self.cursor.row > 0 {
                self.cursor.row -= 1;
                self.cursor.col = self.text.line(self.cursor.row).len_chars();
            }
        }
    }

    fn cursor_to_byte(&self) -> usize {
        self.text.line_to_char(self.cursor.row) + self.cursor.col
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Insert(c) => {
                self.insert_char(c);
            }
            Action::Delete => {
                self.delete_char();
            }
            Action::DeleteFromCommand => {
                self.command_input.pop();
            }
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),
            Action::MoveLeft => self.move_left(),
            Action::MoveRight => self.move_right(),
            Action::EnterInsertMode => self.mode = Mode::Insert,
            Action::EnterNormalMode => self.mode = Mode::Normal,
            Action::EnterVisualMode => self.mode = Mode::Visual,
            Action::EnterCommandMode => {
                self.mode = Mode::Command;
                self.command_input.clear();
            }
            Action::ExecuteCommand => self.execute_command(),
            Action::NoOp => {}
        }
    }

    fn execute_command(&mut self) {
        let command = self.command_input.trim();
        match command {
            "q" | "quit" => self.should_quit = true,
            "" => {}
            _ => {}
        }
        self.mode = Mode::Normal;
        self.command_input.clear();
    }

    pub fn insert_into_command(&mut self, c: char) {
        self.command_input.push(c);
    }
}
