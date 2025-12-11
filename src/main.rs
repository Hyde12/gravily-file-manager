use std::io;

mod gravily;
mod ui;

use gravily::FileManager;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = FileManager::default().run(&mut terminal);

    ratatui::restore();

    result
}
