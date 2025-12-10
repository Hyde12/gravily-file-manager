use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, List, ListState, Paragraph, StatefulWidget, Widget, Wrap},
};

use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use std::env::var;
use std::ffi::OsString;
use std::fs::{metadata, read_dir, read_to_string};
use std::io;
use std::path::PathBuf;

use whoami::DesktopEnv;

enum Action {
    NextItem,
    PreviousItem,
    EnterItem,
    ExitItem,
    NavigationInputMode,
    CommandInputMode,
    AddFile,
    DeleteFile,
    RenameFile,
    InputChar,
    Quit,
    None,
}

#[derive(Debug, Default, PartialEq)]
enum InputMode {
    #[default]
    Navigation,
    Command,
    Operation,
}

#[derive(Debug, Default)]
pub struct FileManager {
    path: PathBuf,
    path_items: Vec<OsString>,
    past_states: Vec<usize>,
    input_mode: InputMode,
    input: Input,
    exit: bool,
    state: ListState,
}

impl FileManager {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if whoami::desktop_env() == DesktopEnv::Windows {
            match var("USERPROFILE") {
                Ok(default_path) => self.path = PathBuf::from(default_path),
                Err(e) => eprintln!("Couldn't get USERPROFILE variable: {}", e),
            }
        } else {
            // assuming linux
            self.path = PathBuf::from("/home/");
            self.path.push("/home/");

            match var("USER") {
                Ok(user) => self.path.push(user),
                Err(e) => eprintln!("Couldn't get USER variable: {}", e),
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

    fn get_hovered_dir(&mut self) -> PathBuf {
        if let Some(path_val) = self.state.selected() {
            let cur_path: PathBuf = [&self.path, &PathBuf::from(&self.path_items[path_val])]
                .iter()
                .collect();

            return cur_path;
        }

        PathBuf::from(&self.path)
    }

    pub fn render_peekable_items(&mut self, area: Rect, buf: &mut Buffer) {
        let cur_path = self.get_hovered_dir();
        if cur_path != PathBuf::from(&self.path) {
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
                    } else if metadata.is_file() {
                        if let Ok(file_text) = read_to_string(&cur_path) {
                            Paragraph::new(file_text).block(block).render(area, buf);
                        }
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

    pub fn render_input_text(&mut self, frame: &mut Frame, area: Rect) {
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);

        let block = Block::bordered()
            .title(" Input ")
            .border_set(border::ROUNDED);

        let text = Line::from(self.input.to_string());

        let input_box = Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Left)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((0, scroll as u16));

        frame.render_widget(input_box, area);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let horizontal_area = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .split(frame.area());
        let list_area = horizontal_area[0];

        // Render the file manager widget (which calls render_file_items, etc.)

        // ... your cursor logic remains
        match self.input_mode {
            InputMode::Operation => {
                // Use the now-set self.input_area
                let area = horizontal_area[1];
                let width = area.width.max(3).saturating_sub(3); // Use saturating_sub for safety
                let scroll = self.input.visual_scroll(width as usize);
                let cursor_pos = self.input.visual_cursor().saturating_sub(scroll);

                // +1 for the block's left border, +1 for the block's top border
                let x = area.x + 1 + cursor_pos as u16;
                let y = area.y + 1;

                frame.set_cursor_position((x, y));
            }
            _ => {}
        }

        self.render_input_text(frame, horizontal_area[1]);
        frame.render_widget(self, list_area); // Or better: frame.render_widget(self, list_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        match &event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.handle_key_event(*key_event) {
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
                    Action::NavigationInputMode => {
                        self.input_mode = InputMode::Navigation;
                        self.input.reset();
                    }
                    Action::CommandInputMode => self.input_mode = InputMode::Command,
                    Action::AddFile => self.input_mode = InputMode::Operation,
                    Action::InputChar => {
                        self.input.handle_event(&event);
                    }
                    Action::Quit => self.exit(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn create_file(&mut self, file_name: String) -> io::Result<bool> {
        let new_file_path: PathBuf = [&self.path, &PathBuf::from(file_name)].iter().collect();
        match new_file_path.try_exists() {
            Ok(new_file_exists) => {
                if !new_file_exists {
                    return Ok(false);
                }

                return Ok(true);
            }
            Err(e) => return Err(e),
        }
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        if self.input_mode == InputMode::Navigation {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                KeyCode::Char('j') | KeyCode::Down => Action::NextItem,
                KeyCode::Char('k') | KeyCode::Up => Action::PreviousItem,
                KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => Action::EnterItem,
                KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => Action::ExitItem,
                KeyCode::Char('a') => Action::AddFile,
                KeyCode::Char('!') => Action::CommandInputMode,
                _ => Action::None,
            }
        } else {
            match key.code {
                KeyCode::Esc => Action::NavigationInputMode,
                _ => Action::InputChar,
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
