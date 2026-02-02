use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub is_dir: bool,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct FilePicker {
    pub current_dir: PathBuf,
    pub files: Vec<FileInfo>,
    pub selected_idx: usize,
}

impl FilePicker {
    pub fn new() -> Self {
        Self {
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            files: Vec::new(),
            selected_idx: 0,
        }
    }

    pub fn refresh(&mut self) {
        self.files = std::fs::read_dir(&self.current_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| {
                        let path = entry.path();
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();
                        let is_dir = path.is_dir();
                        FileInfo { name, is_dir, path }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        self.files.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });

        if self.selected_idx >= self.files.len() && !self.files.is_empty() {
            self.selected_idx = self.files.len() - 1;
        }
    }

    pub fn selected_file(&self) -> Option<&FileInfo> {
        self.files.get(self.selected_idx)
    }

    pub fn move_up(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_idx < self.files.len().saturating_sub(1) {
            self.selected_idx += 1;
        }
    }

    pub fn navigate_to_parent(&mut self) -> bool {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh();
            self.selected_idx = 0;
            true
        } else {
            false
        }
    }
}

impl Default for FilePicker {
    fn default() -> Self {
        Self::new()
    }
}
