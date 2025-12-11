use super::FileManager;

use std::fs::File;
use std::fs::metadata;
use std::fs::remove_file;
use std::fs::rename;

use std::path::Path;

use std::path::PathBuf;

impl FileManager {
    pub fn get_hovered_dir(&mut self) -> PathBuf {
        if let Some(path_val) = self.state.selected() {
            let cur_path: PathBuf = [&self.path, &PathBuf::from(&self.path_items[path_val])]
                .iter()
                .collect();

            return cur_path;
        }

        PathBuf::from(&self.path)
    }

    pub fn is_hovering(&self) -> bool {
        if let Some(_) = self.state.selected() {
            return true;
        }

        false
    }

    pub fn enter_hovered_dir(&mut self) {
        if let Some(path_val) = self.state.selected() {
            let new_path: PathBuf = [&self.path, &PathBuf::from(&self.path_items[path_val])]
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
                Err(e) => {
                    self.error = format!(
                        "Error getting metadata of path {}: {}",
                        new_path.display(),
                        e
                    )
                }
            }
        }
    }

    pub fn load_as_ansi(&mut self, path: &PathBuf) {
        Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<unknown>")
            .to_string();
    }

    pub fn exit_dir(&mut self) {
        self.path.pop();

        let past_state = self.past_states.pop();

        if past_state != None {
            self.state.select(past_state);
        } else {
            self.state.select_first();
        }
    }

    pub fn create_file(&mut self) {
        self.error = String::new();

        let file_name = self.input.value_and_reset();
        let new_file_path: PathBuf = [&self.path, &PathBuf::from(&file_name)].iter().collect();

        let new_file = File::create_new(new_file_path);

        match new_file {
            Ok(_new_file) => return,
            Err(e) => self.error = format!("Error making {}: {}", file_name, e.to_string()),
        }
    }

    pub fn rename_file(&mut self) {
        self.error = String::new();

        let file_name = self.input.value_and_reset();
        let hovered_file = self.get_hovered_dir();

        if hovered_file == self.path {
            self.error = format!(
                "Error renaming file {:?}: File must not be current directory.",
                hovered_file,
            );
            return;
        }

        let new_file: PathBuf = [&self.path, &PathBuf::from(file_name)].iter().collect();
        let renamed_file = rename(&hovered_file, new_file);

        match renamed_file {
            Ok(_renamed_file) => return,
            Err(e) => {
                self.error = format!("Error renaming file {:?}: {}", hovered_file, e.to_string())
            }
        }
    }

    pub fn remove_file(&mut self) {
        self.error = String::new();

        let file_dir = self.get_hovered_dir();

        let removed_file = remove_file(&file_dir);

        match removed_file {
            Ok(_removed_file) => return,
            Err(e) => {
                self.error = format!(
                    "Error deleting {}: {}",
                    file_dir.to_str().unwrap(),
                    e.to_string()
                )
            }
        }
    }

    pub fn exit(&mut self) {
        self.exit = true;
    }
}
