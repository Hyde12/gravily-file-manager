use super::FileManager;
use crate::gravily::InputMode;
use crate::gravily::OperationType::{Add, Delete, Rename};

use crate::io;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use tui_input::backend::crossterm::EventHandler;

enum Action {
    // Navigation Commands
    NextItem,
    PreviousItem,
    EnterItem,
    ExitItem,

    // Operation Commands
    AddFile,
    DeleteFile,
    RenameFile,

    // Input Mode Switching
    NavigationInputMode,
    CommandInputMode,

    // Miscellaneous
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
                    // Navigation Handling
                    Action::NextItem => self.state.select_next(),
                    Action::PreviousItem => self.state.select_previous(),
                    Action::EnterItem => self.enter_hovered_dir(),
                    Action::ExitItem => self.exit_dir(),

                    // Operation Handling
                    Action::AddFile => self.input_mode = InputMode::Operation(Add),
                    Action::RenameFile => self.input_mode = InputMode::Operation(Rename),
                    Action::DeleteFile => self.input_mode = InputMode::Confirmation(Delete),

                    // Input Mode Switch Handling
                    Action::NavigationInputMode => {
                        self.input_mode = InputMode::Navigation;
                        self.input.reset();
                    }
                    Action::CommandInputMode => self.input_mode = InputMode::Command,

                    // Miscellaneous
                    Action::Enter => {
                        match &self.input_mode {
                            // Operation Input
                            InputMode::Operation(Add) => {
                                self.input_mode = InputMode::Confirmation(Add)
                            }

                            InputMode::Operation(Rename) => {
                                self.input_mode = InputMode::Confirmation(Rename)
                            }

                            // Operation Confirmation
                            InputMode::Confirmation(op) => {
                                match op {
                                    Add => self.create_file(),
                                    Rename => self.rename_file(),
                                    Delete => self.remove_file(),
                                }
                                self.input_mode = InputMode::Navigation;
                            }

                            _ => {}
                        }
                    }
                    Action::InputChar => {
                        self.input.handle_event(&event);
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
                // Navigation Controls
                KeyCode::Char('j') | KeyCode::Down => Action::NextItem,
                KeyCode::Char('k') | KeyCode::Up => Action::PreviousItem,
                KeyCode::Char('l') | KeyCode::Enter | KeyCode::Right => Action::EnterItem,
                KeyCode::Char('h') | KeyCode::Backspace | KeyCode::Left => Action::ExitItem,

                // Input Mode Controls
                KeyCode::Char('!') => Action::CommandInputMode,

                // Operation Controls
                KeyCode::Char('a') => Action::AddFile,
                KeyCode::Char('r') if self.is_hovering() => Action::RenameFile,
                KeyCode::Char('d') if self.is_hovering() => Action::DeleteFile,

                // Miscellaneous Controls
                KeyCode::Char('x') => Action::CloseMessage,
                KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
                _ => Action::None,
            },

            InputMode::Operation(Add) | InputMode::Operation(Rename) => match key.code {
                KeyCode::Esc => Action::NavigationInputMode,
                KeyCode::Enter => Action::Enter,
                _ => Action::InputChar,
            },

            InputMode::Operation(Delete) | InputMode::Confirmation(_) => match key.code {
                KeyCode::Esc | KeyCode::Char('n') => Action::NavigationInputMode,
                KeyCode::Enter | KeyCode::Char('y') => Action::Enter,
                _ => Action::None,
            },

            _ => Action::None,
        }
    }
}
