use ropey::Rope;

pub enum Action {
    Quit,
    Insert(char),
    Delete,
    NoOp,
}

/// The core editor state.
pub struct Editor {
    pub text: Rope,
    pub cursor: usize,
    pub scroll_offset: usize,
    pub should_quit: bool,
}

impl Editor {
    pub fn new(initial_text: &str) -> Self {
        Self {
            text: Rope::from_str(initial_text),
            cursor: 0,
            scroll_offset: 0,
            should_quit: false,
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::Insert(c) => {
                self.text.insert_char(self.cursor, c);
                self.cursor += 1;
            }
            Action::Delete => {
                if self.cursor > 0 {
                    self.text.remove(self.cursor - 1..self.cursor);
                    self.cursor -= 1;
                }
            }
            Action::NoOp => {}
        }
    }
}
