use ropey::Rope;

/// The core editor state.
pub struct Editor {
    pub text: Rope,
}

impl Editor {
    pub fn new(initial_text: &str) -> Self {
        Self {
            text: Rope::from_str(initial_text),
        }
    }
}
