mod ui;

use ui::logic::App;
use ui::*;

use crossterm::{
    event::{self, KeyCode},
    execute,
    terminal::{self},
};
use std::error::Error;
use std::io;
use std::process::Command;
use tui::{backend::CrosstermBackend, Terminal};

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
                    KeyCode::Right => app.enter_directory()?,
                    KeyCode::Char('q') => break,
                    KeyCode::Left => app.go_back()?,
                    KeyCode::Char('n') => {
                        let selected_file = &app.files[app.selected];
                        if selected_file.ends_with("") {
                            Command::new("nano")
                                .arg(selected_file)
                                .status()
                                .expect("Failed to open editor");
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
