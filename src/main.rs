use std::io;

mod app;
mod ui;

use app::FileManager;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = FileManager::default().run(&mut terminal);

    ratatui::restore();

    result
}
