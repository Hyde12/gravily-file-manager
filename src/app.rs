use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
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

use std::env::var;
use std::ffi::OsString;
use std::fs::{metadata, read_dir};
use std::io;
use std::path::PathBuf;

use whoami::DesktopEnv;

enum Action {
    NextItem,
    PreviousItem,
    EnterItem,
    ExitItem,
    Quit,
    None,
}

#[derive(Debug, Default)]
pub struct FileManager {
    path: PathBuf,
    path_items: Vec<OsString>,
    past_states: Vec<usize>,
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

    pub fn render_file_items(&mut self, area: Rect, buf: &mut Buffer) {
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
                entry_result
                    .ok()
                    .map(|entry| entry.path().file_name().unwrap().to_owned()) // direntry -> pathbuf -> osstr
            })
            .collect();

        let items: Vec<String> = self
            .path_items
            .iter()
            .map(|file_name| String::from(file_name.to_str().unwrap()))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_symbol(">    ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }

    pub fn render_peekable_items(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(path_val) = self.state.selected() {
            let cur_path: PathBuf = [&self.path, &PathBuf::from(&self.path_items[path_val])]
                .iter()
                .collect();

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
                match self.handle_key_event(key_event) {
                    Action::NextItem => self.state.select_next(),
                    Action::PreviousItem => self.state.select_previous(),
                    Action::EnterItem => {
                        if let Some(path_val) = self.state.selected() {
                            let new_path: PathBuf =
                                [&self.path, &PathBuf::from(&self.path_items[path_val])]
                                    .iter()
                                    .collect();

                            match metadata(&new_path) {
                                Ok(metadata) => {
                                    if metadata.is_dir() {
                                        self.path.push(&PathBuf::from(&self.path_items[path_val]));
                                        self.past_states.push(path_val);
                                        self.state.select_first();
                                    }
                                }
                                Err(e) => eprintln!(
                                    "Error getting metadata of path: {}\n{}",
                                    new_path.display(),
                                    e
                                ),
                            }
                        }
                    }
                    Action::ExitItem => {
                        self.path.pop();

                        let past_state = self.past_states.pop();

                        if past_state != None {
                            self.state.select(past_state);
                        } else {
                            self.state.select_first();
                        }
                    }
                    Action::Quit => self.exit(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('s') | KeyCode::Down => Action::NextItem,
            KeyCode::Char('w') | KeyCode::Up => Action::PreviousItem,
            KeyCode::Enter | KeyCode::Right => Action::EnterItem,
            KeyCode::Backspace | KeyCode::Left => Action::ExitItem,
            _ => Action::None,
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
