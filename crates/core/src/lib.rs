use ropey::Rope;
use std::path::PathBuf;

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
    BufferList,
    SaveDialog,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub id: usize,
    pub text: Rope,
    pub path: Option<PathBuf>,
    pub title: String,
    pub dirty: bool,
    pub is_transient: bool,
}

#[derive(Debug)]
pub struct BufferManager {
    buffers: Vec<Buffer>,
    current_buffer_id: usize,
    next_id: usize,
}

impl BufferManager {
    pub fn new() -> Self {
        let initial_buffer = Buffer {
            id: 0,
            text: Rope::from_str(""),
            path: None,
            title: "[No Name]".to_string(),
            dirty: false,
            is_transient: false,
        };
        Self {
            buffers: vec![initial_buffer],
            current_buffer_id: 0,
            next_id: 1,
        }
    }

    pub fn new_buffer(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let buffer = Buffer {
            id,
            text: Rope::from_str(""),
            path: None,
            title: format!("[Buffer {}]", id),
            dirty: false,
            is_transient: false,
        };

        self.buffers.push(buffer);
        id
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<usize, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(&path)?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("[Untitled]");

        let id = self.next_id;
        self.next_id += 1;

        let buffer = Buffer {
            id,
            text: Rope::from_str(&contents),
            path: Some(path.clone()),
            title: title.to_string(),
            dirty: false,
            is_transient: false,
        };

        self.buffers.push(buffer);
        Ok(id)
    }

    pub fn current_buffer(&self) -> &Buffer {
        self.buffers
            .iter()
            .find(|b| b.id == self.current_buffer_id)
            .expect("Current buffer should exist")
    }

    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        self.buffers
            .iter_mut()
            .find(|b| b.id == self.current_buffer_id)
            .expect("Current buffer should exist")
    }

    pub fn switch_to(&mut self, id: usize) -> bool {
        if self.buffers.iter().any(|b| b.id == id) {
            if let Some(current) = self.buffers.iter().find(|b| b.id == self.current_buffer_id)
                && current.is_transient
            {
                self.delete_buffer(self.current_buffer_id);
            }
            self.current_buffer_id = id;
            true
        } else {
            false
        }
    }

    pub fn next_buffer(&mut self) -> Option<usize> {
        let current_idx = self
            .buffers
            .iter()
            .position(|b| b.id == self.current_buffer_id)?;

        let next_idx = (current_idx + 1) % self.buffers.len();
        Some(self.buffers[next_idx].id)
    }

    pub fn prev_buffer(&mut self) -> Option<usize> {
        let current_idx = self
            .buffers
            .iter()
            .position(|b| b.id == self.current_buffer_id)?;

        let prev_idx = if current_idx == 0 {
            self.buffers.len() - 1
        } else {
            current_idx - 1
        };
        Some(self.buffers[prev_idx].id)
    }

    pub fn delete_buffer(&mut self, id: usize) -> bool {
        if let Some(pos) = self.buffers.iter().position(|b| b.id == id) {
            self.buffers.remove(pos);

            if self.buffers.is_empty() {
                self.buffers.push(Buffer {
                    id: self.next_id,
                    text: Rope::from_str(""),
                    path: None,
                    title: "[No Name]".to_string(),
                    dirty: false,
                    is_transient: false,
                });
                self.next_id += 1;
                self.current_buffer_id = 0;
            } else if self.current_buffer_id == id {
                self.current_buffer_id = self.buffers[0].id;
            }
            true
        } else {
            false
        }
    }

    pub fn save_current(
        &mut self,
        path: Option<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = self.current_buffer_mut();
        let data = buffer.text.to_string();
        let save_path = path.unwrap_or_else(|| {
            buffer
                .path
                .as_ref()
                .map(|p| {
                    if let Some(parent) = p.parent() {
                        if let Some(filename) = p.file_name() {
                            parent.join(filename)
                        } else {
                            p.clone()
                        }
                    } else {
                        PathBuf::from(format!("untitled_{}.txt", buffer.id))
                    }
                })
                .unwrap_or_else(|| PathBuf::from(format!("untitled_{}.txt", buffer.id)))
        });

        std::fs::write(&save_path, data.as_bytes())?;

        buffer.dirty = false;
        buffer.path = Some(save_path.clone());
        buffer.title = save_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("[Untitled]")
            .to_string();

        Ok(())
    }

    pub fn list_buffers(&self) -> Vec<&Buffer> {
        self.buffers.iter().filter(|b| !b.is_transient).collect()
    }

    pub fn current_buffer_id(&self) -> usize {
        self.current_buffer_id
    }
}

impl Default for BufferManager {
    fn default() -> Self {
        Self::new()
    }
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
    SwitchBuffer(usize),
    NextBuffer,
    PrevBuffer,
    ListBuffers,
    SaveBuffer,
    SaveBufferAs(Option<PathBuf>),
    OpenFile(String),
    CancelDialog,
}

/// The core editor state.
pub struct Editor {
    pub buffer_manager: BufferManager,
    pub cursor: Cursor,
    pub scroll_offset: usize,
    pub should_quit: bool,
    pub mode: Mode,
    pub command_input: String,
}

impl Editor {
    pub fn new(_initial_text: &str) -> Self {
        Self {
            buffer_manager: BufferManager::new(),
            cursor: Cursor::new(0, 0),
            scroll_offset: 0,
            should_quit: false,
            mode: Mode::Normal,
            command_input: String::new(),
        }
    }

    pub fn get_current_text(&self) -> &Rope {
        &self.buffer_manager.current_buffer().text
    }

    pub fn get_current_title(&self) -> &str {
        self.buffer_manager.current_buffer().title.as_str()
    }

    pub fn is_current_dirty(&self) -> bool {
        self.buffer_manager.current_buffer().dirty
    }

    pub fn get_current_path(&self) -> Option<&PathBuf> {
        self.buffer_manager.current_buffer().path.as_ref()
    }

    pub fn get_buffers(&self) -> Vec<&Buffer> {
        self.buffer_manager.list_buffers()
    }

    fn cursor_to_byte(&self) -> usize {
        let buffer = self.buffer_manager.current_buffer();
        buffer.text.line_to_char(self.cursor.row) + self.cursor.col
    }

    fn clamp_col_to_line(&mut self) {
        let buffer = self.buffer_manager.current_buffer();
        let line_len = buffer.text.line(self.cursor.row).len_chars();
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.clamp_col_to_line();
        }
    }

    fn move_down(&mut self) {
        let buffer = self.buffer_manager.current_buffer();
        if self.cursor.row < buffer.text.len_lines().saturating_sub(1) {
            self.cursor.row += 1;
            self.clamp_col_to_line();
        }
    }

    fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            let buffer = self.buffer_manager.current_buffer();
            self.cursor.col = buffer.text.line(self.cursor.row).len_chars();
        }
    }

    fn move_right(&mut self) {
        let buffer = self.buffer_manager.current_buffer();
        let line_len = buffer.text.line(self.cursor.row).len_chars();
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        } else if self.cursor.row < buffer.text.len_lines().saturating_sub(1) {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Insert(c) => {
                let byte_pos = self.cursor_to_byte();
                let buffer = self.buffer_manager.current_buffer_mut();
                buffer.text.insert_char(byte_pos, c);
                buffer.dirty = true;
                self.update_cursor_after_insert(c);
            }
            Action::Delete => {
                let byte_pos = self.cursor_to_byte();
                if byte_pos > 0 {
                    let buffer = self.buffer_manager.current_buffer_mut();
                    buffer.text.remove(byte_pos - 1..byte_pos);
                    buffer.dirty = true;
                    self.update_cursor_after_delete();
                }
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
            Action::SwitchBuffer(id) => {
                if self.buffer_manager.switch_to(id) {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Action::NextBuffer => {
                if let Some(id) = self.buffer_manager.next_buffer()
                    && self.buffer_manager.switch_to(id)
                {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Action::PrevBuffer => {
                if let Some(id) = self.buffer_manager.prev_buffer()
                    && self.buffer_manager.switch_to(id)
                {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Action::ListBuffers => {
                self.mode = Mode::BufferList;
            }
            Action::SaveBuffer => {
                let buffer = self.buffer_manager.current_buffer();
                if buffer.path.is_none() {
                    self.mode = Mode::SaveDialog;
                    self.command_input.clear();
                } else if let Err(e) = self.buffer_manager.save_current(None) {
                    eprintln!("Failed to save buffer: {}", e);
                }
            }
            Action::SaveBufferAs(path) => {
                if let Err(e) = self.buffer_manager.save_current(path) {
                    eprintln!("Failed to save buffer: {}", e);
                }
            }
            Action::OpenFile(filename) => {
                let path = PathBuf::from(filename);
                if let Ok(id) = self.buffer_manager.open_file(path)
                    && self.buffer_manager.switch_to(id)
                {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Action::CancelDialog => {
                self.mode = Mode::Normal;
                self.command_input.clear();
            }
            Action::NoOp => {}
        }
    }

    fn update_cursor_after_insert(&mut self, c: char) {
        if c == '\n' {
            self.cursor.row += 1;
            self.cursor.col = 0;
        } else {
            self.cursor.col += 1;
        }
    }

    fn update_cursor_after_delete(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            let buffer = self.buffer_manager.current_buffer_mut();
            self.cursor.col = buffer.text.line(self.cursor.row).len_chars();
        }
    }

    fn execute_command(&mut self) {
        let command = self.command_input.trim();
        let parts: Vec<&str> = command.split_whitespace().collect();

        match parts.first().copied() {
            Some("q") | Some("quit") => self.should_quit = true,
            Some("w") => {
                if let Some(path) = parts.get(1) {
                    let save_path = PathBuf::from(*path);
                    if let Err(e) = self.buffer_manager.save_current(Some(save_path)) {
                        eprintln!("Failed to save buffer: {}", e);
                    }
                } else {
                    let buffer = self.buffer_manager.current_buffer();
                    if buffer.path.is_none() {
                        self.mode = Mode::SaveDialog;
                        self.command_input.clear();
                        return;
                    }
                    if let Err(e) = self.buffer_manager.save_current(None) {
                        eprintln!("Failed to save buffer: {}", e);
                    }
                }
            }
            Some("wq") => {
                let buffer = self.buffer_manager.current_buffer();
                if buffer.path.is_none() {
                    self.mode = Mode::SaveDialog;
                    self.command_input.clear();
                    return;
                }
                if let Err(e) = self.buffer_manager.save_current(None) {
                    eprintln!("Failed to save buffer: {}", e);
                }
                self.should_quit = true;
            }
            Some("!q") => self.should_quit = true,
            Some("b") => {
                self.mode = Mode::BufferList;
                self.command_input.clear();
                return;
            }
            Some("bn") | Some("bnext") => {
                if let Some(id) = self.buffer_manager.next_buffer() {
                    let _ = self.buffer_manager.switch_to(id);
                }
            }
            Some("bp") | Some("bprev") => {
                if let Some(id) = self.buffer_manager.prev_buffer() {
                    let _ = self.buffer_manager.switch_to(id);
                }
            }
            Some("e") => {
                if let Some(filename) = parts.get(1) {
                    let path = PathBuf::from(*filename);
                    if let Ok(id) = self.buffer_manager.open_file(path) {
                        let _ = self.buffer_manager.switch_to(id);
                        self.cursor = Cursor::new(0, 0);
                    }
                }
            }
            Some(n) => {
                if n.len() == 1
                    && let Ok(id) = n.parse::<usize>()
                    && self.buffer_manager.switch_to(id)
                {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            None => {}
        }
        self.mode = Mode::Normal;
        self.command_input.clear();
    }

    pub fn insert_into_command(&mut self, c: char) {
        self.command_input.push(c);
    }
}
