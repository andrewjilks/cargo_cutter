// editor.rs
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout, Write};
use std::path::PathBuf;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub struct TextEditor {
    content: Vec<String>,
    file_path: PathBuf,
    cursor_position: (usize, usize), // (row, column)
    modified: bool,
    theme: ThemeConfig,
}

impl TextEditor {
    pub fn new(file_path: PathBuf, theme: ThemeConfig) -> Self {
        Self {
            content: Vec::new(),
            file_path,
            cursor_position: (0, 0),
            modified: false,
            theme,
        }
    }

    pub fn load_file(&mut self) -> Result<(), String> {
        let content = std::fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        self.content = content.lines().map(|s| s.to_string()).collect();
        if self.content.is_empty() {
            self.content.push(String::new());
        }
        
        self.modified = false;
        Ok(())
    }

    pub fn save_file(&self) -> Result<(), String> {
        let content = self.content.join("\n");
        std::fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn run_editor(&mut self) -> Result<(), String> {
        // Load file content
        self.load_file()?;

        // Setup terminal
        enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
        execute!(stdout(), EnterAlternateScreen).map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

        let result = self.editor_loop();

        // Cleanup terminal
        disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
        execute!(stdout(), LeaveAlternateScreen).map_err(|e| format!("Failed to leave alternate screen: {}", e))?;

        result
    }

    fn editor_loop(&mut self) -> Result<(), String> {
        loop {
            self.draw_interface()?;

            if let Event::Key(key_event) = event::read().map_err(|e| format!("Failed to read event: {}", e))? {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                        if self.modified {
                            // TODO: Add save confirmation in future steps
                            self.save_file()?;
                        }
                        break;
                    }
                    (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                        self.save_file()?;
                        self.modified = false;
                    }
                    _ => {
                        // TODO: Handle other key events in future steps
                    }
                }
            }
        }
        Ok(())
    }

    fn draw_interface(&self) -> Result<(), String> {
        // Clear screen
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
            .map_err(|e| format!("Failed to clear screen: {}", e))?;

        // Draw status bar
        self.draw_status_bar()?;

        // Draw content
        self.draw_content()?;

        // Position cursor
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(self.cursor_position.0 as u16, self.cursor_position.1 as u16)
        ).map_err(|e| format!("Failed to move cursor: {}", e))?;

        stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
        Ok(())
    }

    fn draw_status_bar(&self) -> Result<(), String> {
        let status = if self.modified {
            format!("{} [Modified]", self.file_path.display())
        } else {
            format!("{}", self.file_path.display())
        };

        // Move to bottom of screen and draw status bar
        let terminal_size = crossterm::terminal::size()
            .map_err(|e| format!("Failed to get terminal size: {}", e))?;
        
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(0, terminal_size.1 - 1)
        ).map_err(|e| format!("Failed to move cursor to status bar: {}", e))?;

        AnsiTheme::print_themed(&status, &self.theme);
        Ok(())
    }

    fn draw_content(&self) -> Result<(), String> {
        // Simple content display for Step 1
        for (i, line) in self.content.iter().enumerate() {
            execute!(
                stdout(),
                crossterm::cursor::MoveTo(0, i as u16)
            ).map_err(|e| format!("Failed to move cursor for line {}: {}", i, e))?;
            
            AnsiTheme::print_themed(line, &self.theme);
        }
        Ok(())
    }
}

// Public interface function
pub fn open_file_in_editor(file_path: PathBuf, theme: ThemeConfig) -> Result<(), String> {
    let mut editor = TextEditor::new(file_path, theme);
    editor.run_editor()
}
