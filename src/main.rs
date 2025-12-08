use std::env::var;
use std::fs::{ReadDir, metadata, read_dir};
use std::io;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::{List, ListState, Paragraph, StatefulWidget};
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
    path_items: Vec<PathBuf>,
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

        self.path_items = read_dir(&self.path)
            .unwrap_or_else(|e| {
                panic!("Failed to read directory: {}", e);
            })
            .filter_map(|entry_result| {
                entry_result.ok().map(|entry| entry.path()) // direntry -> pathbuf
            })
            .collect();

        let items: Vec<String> = self
            .path_items
            .iter()
            .filter_map(|path_buf| path_buf.strip_prefix(&self.path).ok())
            .filter_map(|path_buf| path_buf.to_str().map(str::to_owned)) // pathbuf to string
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">    ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    fn render_peekable_items(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(path_val) = self.state.selected() {
            let cur_path: PathBuf = [&self.path, &self.path_items[path_val]].iter().collect();

            let block = Block::bordered()
                .title(Line::from(vec![
                    " ".into(),
                    cur_path.to_str().unwrap().to_string().yellow(),
                    " ".into(),
                ]))
                .border_set(border::ROUNDED);

            match metadata(&cur_path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        match read_dir(&cur_path) {
                            Ok(path) => {
                                let items: Vec<String> = path
                                    .filter_map(|entry_result| entry_result.ok())
                                    .filter_map(|entry| {
                                        let path_buf = entry.path();

                                        path_buf
                                            .strip_prefix(&cur_path)
                                            .ok()
                                            .map(|relative_path| {
                                                relative_path.to_str().map(str::to_owned)
                                            })
                                            .flatten()
                                    })
                                    .collect();
                                let list = List::new(items).block(block);
                                Widget::render(list, area, buf);
                            }
                            Err(e) => {
                                Paragraph::new(format!(
                                    "Failed to read directory\n > {}\n\nError: {}",
                                    &cur_path.display(),
                                    e
                                ))
                                .block(block)
                                .render(area, buf);
                            }
                        }
                    } else {
                        block.render(area, buf);
                    }
                }
                Err(e) => eprintln!(
                    "Error getting metadata of path: {}\n{}",
                    cur_path.display(),
                    e
                ),
            }
        }
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
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('s') | KeyCode::Down => self.select_next(),
            KeyCode::Char('w') | KeyCode::Up => self.select_previous(),
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

        let inner_area = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_block.inner(area));

        main_block.render(area, buf);

        self.render_file_items(inner_area[0], buf);
        self.render_peekable_items(inner_area[1], buf);
        // right_block.render(inner_area[1], buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = FileManager::default().run(&mut terminal);

    ratatui::restore();

    result
}
