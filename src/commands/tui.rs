use std::io;
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::error::Result;
use crate::parser::EnvFile;

pub fn run(files: &[PathBuf]) -> Result<()> {
    if files.is_empty() {
        eprintln!("Usage: envcheck tui <file1.env> <file2.env>");
        return Ok(());
    }

    // Parse all files
    let mut env_files = Vec::new();
    for path in files {
        env_files.push((path.clone(), EnvFile::parse(path)?));
    }

    // Collect all unique keys
    let mut all_keys: Vec<String> = env_files
        .iter()
        .flat_map(|(_, ef)| ef.vars.iter().map(|v| v.key.clone()))
        .collect();
    all_keys.sort();
    all_keys.dedup();

    // Run TUI
    run_tui(&env_files, &all_keys)
        .map_err(|e| crate::error::EnvCheckError::read_error("tui", e))?;

    Ok(())
}

fn run_tui(env_files: &[(PathBuf, EnvFile)], keys: &[String]) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        terminal.draw(|frame| {
            let area = frame.area();

            // Split into header and main content
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            // Header
            let header = Paragraph::new("envcheck TUI - Compare .env files")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(header, chunks[0]);

            // Main content - split horizontally
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Percentage(100 / env_files.len() as u16);
                    env_files.len()
                ])
                .split(chunks[1]);

            // Render each file's keys
            for (i, (path, env_file)) in env_files.iter().enumerate() {
                let items: Vec<ListItem<'_>> = keys
                    .iter()
                    .map(|key| {
                        let value = env_file.vars.iter().find(|v| &v.key == key);
                        let display = match value {
                            Some(v) => format!("{key} = {}", v.value),
                            None => format!("{key} = <missing>"),
                        };
                        let style = if value.is_none() {
                            Style::default().fg(Color::Red)
                        } else {
                            Style::default().fg(Color::Green)
                        };
                        ListItem::new(display).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .title(
                                path.file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                            )
                            .borders(Borders::ALL),
                    )
                    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

                frame.render_stateful_widget(list, main_chunks[i], &mut list_state.clone());
            }

            // Footer with help
            let footer = Paragraph::new("↑/↓: Navigate | q: Quit | Enter: Select")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => {
                            let i = list_state.selected().unwrap_or(0);
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        },
                        KeyCode::Down => {
                            let i = list_state.selected().unwrap_or(0);
                            if i < keys.len().saturating_sub(1) {
                                list_state.select(Some(i + 1));
                            }
                        },
                        _ => {},
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
