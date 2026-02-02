use ropey::Rope;
use std::path::PathBuf;

mod buffer;
mod cursor;
mod file_picker;
mod mode;

pub use buffer::{Buffer, BufferManager};
pub use cursor::Cursor;
pub use file_picker::{FileInfo, FilePicker};
pub use mode::Mode;

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
    CancelKeySequence,
    EnterInsertMode,
    EnterNormalMode,
    EnterVisualMode,
    EnterCommandMode,
    ExecuteCommand,
    SwitchBuffer(usize),
    NextBuffer,
    PrevBuffer,
    CloseBuffer,
    CloseAllBuffersExcept,
    SaveBuffer,
    SaveBufferAs(Option<PathBuf>),
    EnterFilePicker,
    SelectFile(String),
    FilePickerUp,
    FilePickerDown,
    FilePickerEnter,
    FilePickerEsc,
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
    pub file_picker: FilePicker,
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
            file_picker: FilePicker::new(),
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
            Action::CloseBuffer => {
                if let Some(_new_id) = self.buffer_manager.delete_current() {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Action::CloseAllBuffersExcept => {
                let current_id = self.buffer_manager.current_buffer_id();
                self.buffer_manager.delete_all_except(current_id);
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
            Action::CancelKeySequence => {}
            Action::EnterFilePicker => {
                self.mode = Mode::FilePicker;
                self.init_file_picker();
            }
            Action::SelectFile(path) => {
                let path_buf = PathBuf::from(path);
                if let Ok(id) = self.buffer_manager.open_file(path_buf)
                    && self.buffer_manager.switch_to(id)
                {
                    self.cursor = Cursor::new(0, 0);
                }
                self.mode = Mode::Normal;
            }
            Action::FilePickerUp => {
                self.file_picker_up();
            }
            Action::FilePickerDown => {
                self.file_picker_down();
            }
            Action::FilePickerEnter => {
                if let Some(file) = self.file_picker.selected_file() {
                    if file.is_dir {
                        self.file_picker.current_dir = file.path.clone();
                        self.file_picker.refresh();
                        self.file_picker.selected_idx = 0;
                    } else if let Ok(id) = self.buffer_manager.open_file(file.path.clone())
                        // INFO: .clone() here can be replaced with something else as we are giving up on perf here.
                        && self.buffer_manager.switch_to(id)
                    {
                        self.cursor = Cursor::new(0, 0);
                        self.mode = Mode::Normal;
                    }
                }
            }
            Action::FilePickerEsc => {
                self.mode = Mode::Normal;
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
            Some("bn") | Some("bnext") => {
                if let Some(id) = self.buffer_manager.next_buffer() {
                    let _ = self.buffer_manager.switch_to(id);
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Some("bp") | Some("bprev") => {
                if let Some(id) = self.buffer_manager.prev_buffer() {
                    let _ = self.buffer_manager.switch_to(id);
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Some("bx") | Some("bc") | Some("bclose") => {
                if let Some(_new_id) = self.buffer_manager.delete_current() {
                    self.cursor = Cursor::new(0, 0);
                }
            }
            Some("baex") | Some("ballbutexcept") => {
                let current_id = self.buffer_manager.current_buffer_id();
                self.buffer_manager.delete_all_except(current_id);
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

    pub fn init_file_picker(&mut self) {
        self.file_picker.refresh();
    }

    pub fn file_picker_up(&mut self) {
        self.file_picker.move_up();
    }

    pub fn file_picker_down(&mut self) {
        self.file_picker.move_down();
    }

    pub fn file_picker_select(&mut self) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        if let Some(file) = self.file_picker.selected_file() {
            Ok(Some(file.path.clone()))
        } else {
            Ok(None)
        }
    }

    pub fn file_picker_navigate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file) = self.file_picker.selected_file()
            && file.is_dir
        {
            self.file_picker.current_dir = file.path.clone();
            self.file_picker.refresh();
            self.file_picker.selected_idx = 0;
        }
        Ok(())
    }
}
