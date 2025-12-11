use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::ListState,
};

use tui_input::Input;

use std::env::var;
use std::ffi::OsString;
use std::io;
use std::path::PathBuf;

use crate::gravily::OperationType::{Add, Delete};
use whoami::DesktopEnv;
use termimage::{Options};

#[derive(Debug, PartialEq)]
enum OperationType {
    Add,
    Delete,
    Rename,
}

#[derive(Debug, Default, PartialEq)]
enum InputMode {
    #[default]
    Navigation,
    Command,
    Operation(OperationType),
}

#[derive(Debug, Default)]
pub struct FileManager {
    path: PathBuf,
    path_items: Vec<OsString>,
    past_states: Vec<usize>,
    input_mode: InputMode,
    input: Input,
    error: String,
    exit: bool,
    state: ListState,
    ops: Option<Options>,
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

        self.state.select_first();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let mut horizontal_area = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(frame.area());

        if self.input_mode != InputMode::Navigation || !self.error.is_empty() {
            horizontal_area = Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Min(3)])
                .split(frame.area());
        }

        match &self.input_mode {
            InputMode::Navigation => {
                if !self.error.is_empty() {
                    self.render_error_text(frame, horizontal_area[1]);
                }
            }

            InputMode::Operation(Add) => {
                let area = horizontal_area[1];
                let width = area.width.max(3).saturating_sub(3);
                let scroll = self.input.visual_scroll(width as usize);
                let cursor_pos = self.input.visual_cursor().saturating_sub(scroll);

                let x = area.x + 1 + cursor_pos as u16;
                let y = area.y + 1;

                frame.set_cursor_position((x, y));
                self.render_input_text(frame, horizontal_area[1]);
            }

            InputMode::Operation(Delete) => {
                self.render_confirmation_text(frame, horizontal_area[1]);
            }

            _ => {}
        }

        let list_area = horizontal_area[0];
        frame.render_widget(self, list_area);
    }
}

mod helper_functions;
mod input_handling;
mod render_handling;
