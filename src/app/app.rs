use std::{io, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    crossterm,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};

use crate::app::util::{find_sln_files, parse_sln_for_projects};

pub struct App {
    pub exit: bool,
    pub sln_files: Vec<String>,
    pub selected_sln: String,
    pub projects: Vec<String>,
    list_state: ListState,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let sln_files = find_sln_files()?;
        let selected_sln = sln_files
            .first()
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "No .sln files found",
            ))?
            .clone();
        let projects = parse_sln_for_projects(&selected_sln)?;

        Ok(Self {
            exit: false,
            sln_files,
            selected_sln,
            projects,
            list_state: ListState::default().with_selected(Some(0)),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        while !self.exit {
            terminal.draw(|f| {
                let items = self.sln_files.iter().map(|path| {
                    let name = std::path::Path::new(path)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    ListItem::new(Span::styled(name, Style::default().fg(Color::Yellow)))
                });

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Solutions (↑/↓: navigate, Enter: select, q: quit) "),
                    )
                    .highlight_style(Style::default().bg(Color::DarkGray))
                    .highlight_symbol("➤ ");

                f.render_stateful_widget(list, f.area(), &mut self.list_state);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Up => self.move_selection(-1),
                            KeyCode::Down => self.move_selection(1),
                            KeyCode::Enter => self.select_solution()?,
                            _ => {}
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        terminal.backend_mut().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn move_selection(&mut self, delta: i32) {
        if let Some(current) = self.list_state.selected() {
            let new = current
                .saturating_add_signed(delta as isize)
                .min(self.sln_files.len() - 1);
            self.list_state.select(Some(new));
        }
    }

    fn select_solution(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            self.selected_sln = self.sln_files[selected].clone();
            self.projects = parse_sln_for_projects(&self.selected_sln)?;
        }
        Ok(())
    }
}
