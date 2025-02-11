use std::{io, path::Path, time::Duration};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{self, event::KeyEventKind, ExecutableCommand},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Terminal,
};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let mut app = App::new()?;
    app.run()?;

    println!("\nSelected solution: {}", app.selected_sln);
    println!("Projects:");
    for project in &app.projects {
        println!("  - {}", project);
    }

    Ok(())
}

struct App {
    exit: bool,
    sln_files: Vec<String>,
    selected_sln: String,
    projects: Vec<String>,
    list_state: ListState,
}

impl App {
    fn new() -> io::Result<Self> {
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

    fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        while !self.exit {
            terminal.draw(|f| {
                let items = self.sln_files.iter().map(|path| {
                    let name = Path::new(path)
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

fn find_sln_files() -> io::Result<Vec<String>> {
    let dir = Path::new(r"C:\Users\Berat Hündürel\Desktop\Software\Personal");
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sln"))
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect())
}

fn parse_sln_for_projects(sln_path: &str) -> io::Result<Vec<String>> {
    Ok(std::fs::read_to_string(sln_path)?
        .lines()
        .filter_map(|line| {
            line.trim()
                .starts_with("Project(")
                .then(|| line.split(',').nth(1))
                .flatten()
                .map(|s| s.trim().trim_matches('"').to_string())
        })
        .collect())
}
