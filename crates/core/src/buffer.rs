use ropey::Rope;
use std::path::PathBuf;

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

    pub fn delete_current(&mut self) -> Option<usize> {
        let current_id = self.current_buffer_id;
        let current_idx = self.buffers.iter().position(|b| b.id == current_id)?;

        self.buffers.remove(current_idx);

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
            Some(0)
        } else {
            let next_idx = current_idx.min(self.buffers.len() - 1);
            self.current_buffer_id = self.buffers[next_idx].id;
            Some(self.current_buffer_id)
        }
    }

    pub fn delete_all_except(&mut self, keep_id: usize) {
        let mut i = 0;
        while i < self.buffers.len() {
            if self.buffers[i].id != keep_id {
                self.buffers.remove(i);
            } else {
                i += 1;
            }
        }
        self.current_buffer_id = keep_id;
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
