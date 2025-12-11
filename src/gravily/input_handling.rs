use super::FileManager;
use crate::gravily::InputMode;
use crate::gravily::OperationType::{Add, Delete};

use crate::io;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tui_input::backend::crossterm::EventHandler;

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
    Enter,
    InputChar,
    CloseMessage,
    Quit,
    None,
}

impl FileManager {
    pub fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        match &event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.handle_key_event(*key_event) {
                    Action::NextItem => self.state.select_next(),
                    Action::PreviousItem => self.state.select_previous(),
                    Action::EnterItem => self.enter_hovered_dir(),
                    Action::ExitItem => self.exit_dir(),
                    Action::NavigationInputMode => {
                        self.input_mode = InputMode::Navigation;
                        self.input.reset();
                    }
                    Action::CommandInputMode => self.input_mode = InputMode::Command,
                    Action::AddFile => self.input_mode = InputMode::Operation(Add),
                    Action::DeleteFile => self.input_mode = InputMode::Operation(Delete),
                    Action::InputChar => {
                        self.input.handle_event(&event);
                    }
                    Action::Enter => {
                        match &self.input_mode {
                            InputMode::Operation(Add) => self.create_file(),
                            InputMode::Operation(Delete) => self.remove_file(),
                            _ => {}
                        }
                        self.input_mode = InputMode::Navigation;
                    }
                    Action::CloseMessage => self.error = String::new(),
                    Action::Quit => self.exit(),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Action {
        match &self.input_mode {
            InputMode::Navigation => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                KeyCode::Char('j') | KeyCode::Down => Action::NextItem,
                KeyCode::Char('k') | KeyCode::Up => Action::PreviousItem,
                KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => Action::EnterItem,
                KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => Action::ExitItem,
                KeyCode::Char('a') => Action::AddFile,
                KeyCode::Char('d') => Action::DeleteFile,
                KeyCode::Char('x') => Action::CloseMessage,
                KeyCode::Char('!') => Action::CommandInputMode,
                _ => Action::None,
            },

            InputMode::Operation(Add) => match key.code {
                KeyCode::Esc => Action::NavigationInputMode,
                KeyCode::Enter => Action::Enter,
                _ => Action::InputChar,
            },

            InputMode::Operation(Delete) => match key.code {
                KeyCode::Esc | KeyCode::Char('n') => Action::NavigationInputMode,
                KeyCode::Char('y') => Action::Enter,
                _ => Action::None,
            },

            _ => Action::None,
        }
    }
}
