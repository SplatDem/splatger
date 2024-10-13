use crate::logic;

use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use ratatui::layout::{Constraint, Direction, Layout};

use logic::*;

pub fn draw_ui<B: Write>(
    terminal: &mut Terminal<CrosstermBackend<B>>,
    app: &App,
) -> Result<(), Box<dyn Error>> {
    terminal.draw(|f| {
        let _size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(8),
                    Constraint::Percentage(82),
                    Constraint::Percentage(9),
                ]
                .as_ref(),
            )
            .split(f.area());

        let chunks2 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(chunks[1]);

        let previous_dir = if let Some(prev) = app.previous_dirs.last() {
            format!("Previous Directory: {}", prev.display())
        } else {
            "Previous Directory: None".to_string()
        };
        let prev_dir_paragraph = Paragraph::new(previous_dir).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Previous Directory"),
        );
        f.render_widget(prev_dir_paragraph, chunks[0]);

        // Blocks
        let block = Block::default().title("").borders(Borders::ALL);
        f.render_widget(block, chunks[0]);

        let block_h = Block::default().title("").borders(Borders::ALL);
        f.render_widget(block_h, chunks2[0]);

        let preview_paragraph = Paragraph::new(app.preview_text.as_str())
            .block(Block::default().borders(Borders::ALL).title("Preview"));
        f.render_widget(preview_paragraph, chunks2[1]);

        let current_file = &app.files[app.selected];
        let mut absolute_current_file = PathBuf::from(&app.current_dir);

        absolute_current_file.push(current_file);

        let file_info = if let Some(path_str) = absolute_current_file.to_str() {
            get_file_info(path_str)
        } else {
            "Unknown path".to_string()
        };

        let info_paragraph = Paragraph::new(file_info)
            .block(Block::default().borders(Borders::ALL).title("File Info"));
        f.render_widget(info_paragraph, chunks[2]);

        // Blocks

        let visible_files = &app.files
            [app.offset..app.offset + app.visible_count.min(app.files.len() - app.offset)];
        let items: Vec<ListItem> = visible_files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let style = if i + app.offset == app.selected {
                    ratatui::style::Style::default().fg(ratatui::style::Color::Green)
                } else {
                    ratatui::style::Style::default().fg(ratatui::style::Color::White)
                };
                ListItem::new(file.as_str()).style(style)
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, chunks2[0]);
    })?;

    Ok(())
}
