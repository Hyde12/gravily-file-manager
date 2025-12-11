# 📁 Gravily File Manager

A fast, keyboard-driven file manager for the terminal, built in **Rust** using the **Ratatui** (TUI) library.

---

## ✨ Features

- **⚡️ Blazing Fast:** Leverages Rust's performance for snappy navigation and operation.
- **⌨️ Vim-like Keybindings:** Efficient navigation and file manipulation using familiar modal editing concepts.
- **📂 Standard Operations:** Create, delete, and rename files.
- **👀 Real-time Previews:** Displays file content (text, image etc.) in a dedicated preview pane.
  
---

## ✅ To-Do / Roadmap

Features and improvements planned for development (from most to least prioritized):

- ~~**File Operations:** Be able to add, edit, rename, within the CLI.~~ _still have to add copying and moving, plus the same operations for directories_
- ~~**Image Preview:** Ability to see image previews in the CLI.~~ _thank you [@venoosoo](https://github.com/venoosoo)_
- **Built-in Shell Execution:** Add a keybinding (e.g., `!`) to run shell commands without exiting.
- **Syntax Highlighting:** Highlight the syntax of certain previewed files.
- **User Customization:** Change the colors, icons, etc., via json file (for easy pywal integration).
- **Asynchronous Operations:** Perform heavy file operations in the background.
- **Bookmark Management:** Save and jump to frequently used directories.
- **Customizable Columns:** Choose which metadata (size, permissions, date) appears in the main pane.

To suggest a feature, open an issue on GitHub with the **feature request** tag.

---

## 🚀 Installation

### Prerequisites

You need **Rust** and **Cargo** installed on your system. If you don't have it, you can install it via [rustup](https://rustup.rs/).

### From Source

1. **Clone the repository:**

   ```bash
   git clone https://github.com/Hyde12/gravily-file-manager
   cd gravily-file-manager
   ```

2. **Build and install:**

   ```bash
   cargo install --path .
   ```

---

## 🖥️ Usage

Simply run the executable from your terminal:

```bash
gravily
```

---

## 🎯 Keybindings

| Action       | Keybinding(s)     | Description                  |
| ------------ | ----------------- | ---------------------------- |
| **Movement** | `j`, `↓`          | Move down one item           |
|              | `k`, `↑`          | Move up one item             |
|              | `h`, `←`          | Go to parent directory       |
|              | `l`, `→`, `Enter` | Open file or enter directory |
| Other        | `q`, `Esc`        | Quit Gravily                 |

---

## 🤝 Contributing

Contributions are welcome!

1. Fork the repository.
2. Create your feature branch:

   ```bash
   git checkout -b feature/AmazingFeature
   ```

3. Commit your changes:

   ```bash
   git commit -m "feat: files bitcoin mine"
   ```

4. Push to the branch:

   ```bash
   git push origin feature/AmazingFeature
   ```

5. Open a Pull Request.

---

## 📄 License

Distributed under the **MIT License**. See the `LICENSE` file for details.

---

## 🔨 Built With

- **Rust** – The programming language.
- **Ratatui** – TUI/terminal interface library.
- **Crossterm** – Terminal control backend.

---

**Author:** [Hyde12](https://github.com/Hyde12)

**Project Link:** [https://github.com/Hyde12/gravily-file-manager](https://github.com/Hyde12/gravily-file-manager)
