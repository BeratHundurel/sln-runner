use std::{
    fs,
    io::{self},
    path::Path,
    process::Command,
    time::Duration,
};

use serde_json::Value;

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
    showing_projects: bool,
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
            showing_projects: false,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        stdout.execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        while !self.exit {
            terminal.draw(|f| {
                if self.showing_projects {
                    self.draw_project_list(f);
                } else {
                    self.draw_solution_list(f);
                }
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => self.exit = true,
                            KeyCode::Up => self.move_selection(-1),
                            KeyCode::Down => self.move_selection(1),
                            KeyCode::Enter => self.on_enter_key()?,
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

    fn draw_solution_list(&mut self, f: &mut ratatui::Frame) {
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
    }

    fn draw_project_list(&mut self, f: &mut ratatui::Frame) {
        let items = self.projects.iter().map(|project| {
            ListItem::new(Span::styled(
                project.clone(),
                Style::default().fg(Color::Yellow),
            ))
        });

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Projects (↑/↓: navigate, Enter: select, q: quit) "),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("➤ ");

        f.render_stateful_widget(list, f.area(), &mut self.list_state);
    }

    fn move_selection(&mut self, delta: i32) {
        if let Some(current) = self.list_state.selected() {
            let max_len = if self.showing_projects {
                self.projects.len()
            } else {
                self.sln_files.len()
            };

            let new = current
                .saturating_add_signed(delta as isize)
                .min(max_len.saturating_sub(1));
            self.list_state.select(Some(new));
        }
    }

    fn on_enter_key(&mut self) -> io::Result<()> {
        if self.showing_projects {
            self.run_selected_project()?;
            return Ok(());
        }

        self.select_solution()?;
        Ok(())
    }

    fn detect_launch_profile(launch_settings_path: &Path) -> Option<String> {
        if let Ok(contents) = fs::read_to_string(launch_settings_path) {
            if let Ok(json) = serde_json::from_str::<Value>(&contents) {
                if let Some(profiles) = json.get("profiles").and_then(|p| p.as_object()) {
                    return profiles.keys().next().cloned(); // Get first profile name
                }
            }
        }
        None
    }

    fn run_selected_project(&self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            let project = &self.projects[selected];

            println!("Running project: {}", project);

            let project_path = std::path::Path::new(&self.selected_sln)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join(project);

            let output = Command::new("dotnet")
                .arg("build")
                .arg("--configuration")
                .arg("Debug")
                .arg(&project_path)
                .output()?;

            if !output.status.success() {
                eprintln!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
                return Err(io::Error::new(io::ErrorKind::Other, "Build failed"));
            }

            println!("Build successful! Running ...");

            let launch_settings_path = project_path.join("Properties").join("launchSettings.json");
            let launch_profile =  Self::detect_launch_profile(&launch_settings_path);

            let mut command = Command::new("dotnet");
            command.arg("run").current_dir(&project_path);

            if let Some(profile) = launch_profile {
                println!("Detected launch profile: {}", profile);
                command.arg("--launch-profile").arg(profile);
            } else {
                println!("No launch profile found, running normally...");
            }

            command.spawn()?;

            return Ok(());
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No project selected",
        ))
    }

    fn select_solution(&mut self) -> io::Result<()> {
        if let Some(selected) = self.list_state.selected() {
            self.selected_sln = self.sln_files[selected].clone();
            self.projects = parse_sln_for_projects(&self.selected_sln)?;
            self.showing_projects = true;
            self.list_state = ListState::default().with_selected(Some(0));
        }
        Ok(())
    }

}
