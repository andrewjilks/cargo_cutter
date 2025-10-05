use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout};
use std::path::PathBuf;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

use super::input;
use super::render;

pub struct TextEditor {
    pub content: Vec<String>,
    pub file_path: PathBuf,
    pub cursor_position: (usize, usize),
    pub viewport_offset: (usize, usize),
    pub modified: bool,
    pub theme: ThemeConfig,
    pub exit_requested: bool,
    pub needs_redraw: bool,
    pub previous_terminal_size: (u16, u16),
    pub desired_column: usize,
    pub clipboard: String,
    pub selection_start: Option<(usize, usize)>,
}

impl TextEditor {
    pub fn new(file_path: PathBuf, theme: ThemeConfig) -> Self {
        Self {
            content: Vec::new(),
            file_path,
            cursor_position: (0, 0),
            viewport_offset: (0, 0),
            modified: false,
            theme,
            exit_requested: false,
            needs_redraw: true,
            previous_terminal_size: (0, 0),
            desired_column: 0,
            clipboard: String::new(),
            selection_start: None,
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
        self.desired_column = self.cursor_position.0;
        self.clear_selection();
        Ok(())
    }

    pub fn save_file(&self) -> Result<(), String> {
        let content = self.content.join("\n");
        std::fs::write(&self.file_path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        Ok(())
    }

    pub fn run_editor(&mut self) -> Result<(), String> {
        self.load_file()?;

        enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
        execute!(stdout(), EnterAlternateScreen).map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

        let result = self.editor_loop();

        disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
        execute!(stdout(), LeaveAlternateScreen).map_err(|e| format!("Failed to leave alternate screen: {}", e))?;

        result
    }

    fn editor_loop(&mut self) -> Result<(), String> {
        loop {
            let current_size = crossterm::terminal::size()
                .map_err(|e| format!("Failed to get terminal size: {}", e))?;
            
            if self.needs_redraw || current_size != self.previous_terminal_size {
                render::draw_interface(self)?;
                self.previous_terminal_size = current_size;
                self.needs_redraw = false;
            }

            if event::poll(std::time::Duration::from_millis(16))
                .map_err(|e| format!("Failed to poll event: {}", e))? 
            {
                if let Event::Key(key_event) = event::read()
                    .map_err(|e| format!("Failed to read event: {}", e))? 
                {
                    if !input::handle_key_event(self, key_event) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some()
    }
}

pub fn open_file_in_editor(file_path: PathBuf, theme: ThemeConfig) -> Result<(), String> {
    let mut editor = TextEditor::new(file_path, theme);
    editor.run_editor()
}
