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
