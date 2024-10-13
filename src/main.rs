mod logic;
mod ui;

use crate::logic::App;
use crate::ui::*;

use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{self},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(".")?; // По хацкерски начинать с текущей директории

    loop {
        draw_ui(&mut terminal, &app)?;

        if event::poll(std::time::Duration::from_millis(500))? {
            if let event::Event::Key(event) = event::read()? {
                match event.code {
                    KeyCode::Up => app.navigate(-1),
                    KeyCode::Down => app.navigate(1),
                    KeyCode::Enter => app.enter_directory()?,
                    KeyCode::Char('q') => break,
                    KeyCode::Left => app.go_back()?,
                    KeyCode::Char('n') => {
                        let selected_file = &app.files[app.selected];
                        let mut absolute_nano = PathBuf::from(&app.current_dir);
                        absolute_nano.push(selected_file);

                        if absolute_nano.ends_with("") {
                            Command::new("nano")
                                .arg(absolute_nano)
                                .status()
                                .expect("Failed to open editor");
                        }
                    }
                    KeyCode::Right => {
                        let _ = app.enter_directory();
                        let selected_file = &app.files[app.selected];

                        let mut absolute_path = PathBuf::from(&app.current_dir);
                        absolute_path.push(selected_file);

                        if absolute_path.exists() {
                            if let Some(extension) = absolute_path.extension() {
                                if extension == "png"
                                    || extension == "jpg"
                                    || extension == "jpeg"
                                    || extension == "gif"
                                    || extension == "mp3"
                                    || extension == "mp4"
                                    || extension == "mkv"
                                    || extension == "flac"
                                {
                                    Command::new("xdg-open")
                                        .arg(absolute_path)
                                        .status()
                                        .expect("Failed to open");
                                } /*else {
                                      println!("what the extension");
                                  }*/
                            }
                        } else {
                            println!("File not found: {:?}", absolute_path);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}
