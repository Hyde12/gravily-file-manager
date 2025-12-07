use std::env::var;
use std::fs::read_dir;
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::{List, ListState, StatefulWidget};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};
use whoami::DesktopEnv;

#[derive(Debug, Default)]
pub struct FileManager {
    path: PathBuf,
    exit: bool,
    state: ListState,
}

impl FileManager {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if whoami::desktop_env() == DesktopEnv::Windows {
            match var("USERPROFILE") {
                Ok(default_path) => {
                    self.path = PathBuf::from(default_path);
                }
                Err(e) => {
                    eprintln!("Couldn't get USERPROFILE variable: {}", e);
                }
            }
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_file_items(&mut self, area: Rect, buf: &mut Buffer) {
        let path_text = Line::from(vec![
            " Current Path: ".into(),
            self.path.to_str().unwrap().to_string().yellow(),
            " ".into(),
        ]);

        let block = Block::bordered()
            .title(path_text)
            .border_set(border::ROUNDED);

        let items: Vec<String> = read_dir(&self.path)
            .unwrap_or_else(|e| {
                panic!("Failed to read directory: {}", e);
            })
            .filter_map(|entry_result| {
                entry_result
                    .ok()
                    .map(|entry| entry.path()) // direntry -> pathbuf
                    .map(|path_buf| match path_buf.strip_prefix(&self.path) {
                        // pathbuf shortened to file/dir name
                        Ok(new_path) => Some(new_path.to_owned()),
                        Err(_) => None,
                    })
                    .flatten()
                    .and_then(|path_buf| path_buf.to_str().map(str::to_owned)) // pathbuf to string
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">    ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Down => self.select_next(),
            KeyCode::Up => self.select_previous(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }

    fn select_previous(&mut self) {
        self.state.select_previous();
    }
}

impl Widget for &mut FileManager {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Gravily File Manager ".bold());
        let instructions = Line::from(vec![
            " Arrow keys ".into(),
            "<Move>".blue().bold(),
            " Enter ".into(),
            "<Enter Dir>".blue().bold(),
            " Quit ".into(),
            "<q or esc> ".blue().bold(),
        ]);
        let main_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::PLAIN);

        let inner_area = main_block.inner(area);

        main_block.render(area, buf);

        let left_area = Rect::new(
            inner_area.x,
            inner_area.y,
            inner_area.width / 2,
            inner_area.height,
        );
        let right_area = Rect::new(
            inner_area.x + inner_area.width / 2,
            inner_area.y,
            inner_area.width / 2,
            inner_area.height,
        );

        let right_block = Block::bordered()
            .title(Line::from(whoami::desktop_env().to_string()))
            .border_set(border::ROUNDED);

        self.render_file_items(left_area, buf);
        right_block.render(right_area, buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = FileManager::default().run(&mut terminal);

    ratatui::restore();

    result
}
