use crossterm::{execute, terminal};

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use walkdir::WalkDir;

pub struct App {
    pub current_dir: PathBuf,
    pub previous_dirs: Vec<PathBuf>,
    pub files: Vec<String>,
    pub selected: usize,
    pub offset: usize,
    pub visible_count: usize,
    pub preview_text: String,
}

pub fn read_file_content<P: AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

pub fn get_file_info(file_path: &str) -> String {
    match fs::metadata(file_path) {
        Ok(metadata) => {
            let file_type = if metadata.is_dir() {
                "Directory"
            } else {
                "File"
            };

            let size = metadata.len();
            let size_display = if size >= 1_000_000 {
                format!("{:.2} MB", size as f64 / 1_048_576.0)
            } else if size >= 1_000 {
                format!("{:.2} KB", size as f64 / 1_024.0)
            } else {
                format!("{} bytes", size)
            };

            let modified_time = match metadata.modified() {
                Ok(time) => time,
                Err(_) => SystemTime::now(),
            };

            let now = SystemTime::now();
            let duration = match now.duration_since(modified_time) {
                Ok(duration) => duration,
                Err(_) => {
                    return format!("Error: The modified time is in the future.");
                }
            };

            let hours = duration.as_secs() / 3600;

            format!(
                "Type: {}\nSize: {}\nModified: {} hours ago",
                file_type, size_display, hours
            )
        }
        Err(e) => format!("Error accessing file: {}", e),
    }
}

impl App {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
        let path = path.as_ref().to_path_buf();
        let files = App::read_files(&path)?;
        Ok(App {
            current_dir: path,
            previous_dirs: Vec::new(),
            files,
            selected: 0,
            offset: 0,
            visible_count: 38,
            preview_text: String::new(),
        })
    }

    pub fn read_files<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
        let files = WalkDir::new(path)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(|entry| {
                if let Ok(e) = entry {
                    Some(e.file_name().to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();
        Ok(files)
    }

    pub fn navigate(&mut self, direction: isize) {
        let count = self.files.len();
        if direction > 0 && self.selected < count - 1 {
            self.selected += 1;
        } else if direction < 0 && self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected >= self.offset + self.visible_count {
            self.offset += 1;
        } else if self.selected < self.offset {
            if self.offset > 0 {
                self.offset -= 1;
            }
        }
    }

    pub fn enter_directory(&mut self) -> Result<(), Box<dyn Error>> {
        if self.selected < self.files.len() {
            let selected_path = &self.files[self.selected];
            let new_path = self.current_dir.join(selected_path);
            if new_path.is_dir() {
                self.previous_dirs.push(self.current_dir.clone());
                self.current_dir = new_path.to_path_buf();
                self.files = App::read_files(&self.current_dir)?;
                self.selected = 0;
                self.preview_text.clear();
            } else if selected_path.ends_with("") {
                self.preview_text = read_file_content(new_path)?;
            } else {
                self.preview_text.clear();
            }
        }
        Ok(())
    }

    pub fn go_back(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(prev_dir) = self.previous_dirs.pop() {
            self.current_dir = prev_dir;
            self.files = App::read_files(&self.current_dir)?;
            self.selected = 0;
        }
        Ok(())
    }
}
