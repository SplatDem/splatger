use crossterm::{
    event::{self, KeyCode},
    terminal::{self, ClearType},
    execute,
};
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::cell::Ref;
use std::process::Command;
use std::fs;
use tui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

use walkdir::{WalkDir, DirEntry};

struct App {
    current_dir: PathBuf,
    previous_dirs: Vec<PathBuf>,
    files: Vec<String>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    preview_text: String,
}

fn read_file_content<P:AsRef<Path>>(path: P) -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}

fn get_style_by_extension(file: &str) -> tui::style::Style {
    let extension = file.split('.').last().unwrap_or("");
    match extension {
        "rs" => tui::style::Style::default().fg(tui::style::Color::Rgb(255,127,80)),
        "txt" => tui::style::Style::default().fg(tui::style::Color::Green),
        "jpg" | "jpeg" | "png" => tui::style::Style::default().fg(tui::style::Color::Yellow),
        "mp3" | "wav" => tui::style::Style::default().fg(tui::style::Color::Magenta),
        "c" => tui::style::Style::default().fg(tui::style::Color::Gray),
        _ => tui::style::Style::default().fg(tui::style::Color::White),
    }
}

fn get_file_info(file_path: &str) -> String {
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let file_type = if metadata.is_dir() { "Directory" } else { "File" };
            let size = metadata.len();
            let modified_time = metadata.modified().unwrap_or_else(|_| std::time::SystemTime::now()).duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
            
            format!("Type: {}\nSize: {} bytes\nModified: {} seconds ago", file_type, size, modified_time)
        },
        Err(e) => format!("Error: {}", e),
    }
}

impl App {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
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

    fn read_files<P: AsRef<Path>>(path: P) -> Result<Vec<String>, Box<dyn Error>> {
    let files = WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(e) = entry {
                Some(e.file_name().to_string_lossy().into_owned()) // Возвращаем только имя файла/директории
            } else {
                None
            }
        })
        .collect();
    Ok(files)
    }

    
    fn navigate(&mut self, direction: isize) {
        let count = self.files.len();
        // self.selected = (self.selected as isize + direction).rem_euclid(count as isize) as usize;
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

    fn enter_directory(&mut self) -> Result<(), Box<dyn Error>> {
        if self.selected < self.files.len() {
            let selected_path = &self.files[self.selected];
            let new_path = self.current_dir.join(selected_path);
            if new_path.is_dir() {
                self.previous_dirs.push(self.current_dir.clone());
                self.current_dir = new_path.to_path_buf();
                self.files = App::read_files(&self.current_dir)?;
                self.selected = 0;  // Сбрасываем выделение
                self.preview_text.clear();
            } else if selected_path.ends_with("") {
                self.preview_text = read_file_content(new_path)?;
            } else {
                self.preview_text.clear();
            }
        }
        Ok(())
    }

    fn go_back(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(prev_dir) = self.previous_dirs.pop() {
            self.current_dir = prev_dir;
            self.files = App::read_files(&self.current_dir)?;
            self.selected = 0; // Сбрасываем выделение
        }
        Ok(())
    }
}

fn draw_ui<B: Write>(terminal: &mut Terminal<CrosstermBackend<B>>, app: &App) -> Result<(), Box<dyn Error>> {
    terminal.draw(|f| {
        let size = f.size();

        let chunks = Layout::default().direction(Direction::Vertical).margin(1).constraints(
                [
                    Constraint::Percentage(8),
                    Constraint::Percentage(82),
                    Constraint::Percentage(9)
                ].as_ref()

            )
            .split(f.size());
        let chunks2 = Layout::default().direction(Direction::Horizontal).margin(0).constraints(
                [ Constraint::Percentage(25), Constraint::Percentage(60), Constraint::Percentage(30) ].as_ref()
            ).split(chunks[1]);
       

        let previous_dir = if let Some(prev) = app.previous_dirs.last() {
            format!("Previous Directory: {}", prev.display())
        } else {
            "Previous Directory: None".to_string()
        };
        let prev_dir_paragraph = Paragraph::new(previous_dir)
            .block(Block::default().borders(Borders::ALL).title("Previous Directory"));
        f.render_widget(prev_dir_paragraph, chunks[0]);

        // Blocks
        let block = Block::default().title("").borders(Borders::ALL);
        f.render_widget(block, chunks[0]);

        let blockH = Block::default().title("").borders(Borders::ALL);
        f.render_widget(blockH, chunks2[0]);

        let blockH = Block::default().title("").borders(Borders::ALL);
        f.render_widget(blockH, chunks2[2]);

        let preview_paragraph = Paragraph::new(app.preview_text.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Preview"));
        f.render_widget(preview_paragraph, chunks2[1]);


        let current_file = &app.files[app.selected];
        let file_info = get_file_info(current_file);
        let info_paragraph = Paragraph::new(file_info)
            .block(Block::default().borders(Borders::ALL).title("File Info"));
        f.render_widget(info_paragraph, chunks[2]);
        // Blocks
        
        let visible_files = &app.files[app.offset..app.offset + app.visible_count.min(app.files.len() - app.offset)];
        let items: Vec<ListItem> = visible_files.iter()
            .enumerate()
            .map(|(i, file)| {
                let style = if i + app.offset == app.selected {
                    tui::style::Style::default().fg(tui::style::Color::Green)
                } else {
                    tui::style::Style::default().fg(tui::style::Color::White)
                };
                ListItem::new(file.as_str()).style(style)
            }).collect();
        
        let list = List::new(items);
        f.render_widget(list, chunks2[0]);
    })?;
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    terminal::enable_raw_mode()?;
    
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new(".")?;  // По хацкерски начинать с текущей директории

    loop {
        draw_ui(&mut terminal, &app)?;

        if event::poll(std::time::Duration::from_millis(500))? {
            if let event::Event::Key(event) = event::read()? {
                match event.code {
                    KeyCode::Up => app.navigate(-1),
                    KeyCode::Down => app.navigate(1),
                    KeyCode::Enter => app.enter_directory()?, KeyCode::Right => app.enter_directory()?,
                    KeyCode::Char('q') => break, // Выход из приложения
                    KeyCode::Left => app.go_back()?,
//                    KeyCode::F(1) => app.open_terminal()?,
                    KeyCode::Char('n') => {
                        let selected_file = &app.files[app.selected];
                         if selected_file.ends_with("") {
                            Command::new("nano")
                                .arg(selected_file).status().expect("Failed to open editor");
                         }
                    },
                    _ => {},
                }
            }
        }
    }

    execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

