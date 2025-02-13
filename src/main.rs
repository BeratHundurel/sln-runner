mod app;

use app::App;
use std::io;

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
