use std::{io, path::Path};
use walkdir::WalkDir;

pub fn find_sln_files() -> io::Result<Vec<String>> {
    let dir = Path::new(r"C:\Users\Berat Hündürel\Desktop\Software\Personal");
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "sln"))
        .map(|e| e.path().to_string_lossy().into_owned())
        .collect())
}

pub fn parse_sln_for_projects(sln_path: &str) -> io::Result<Vec<String>> {
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