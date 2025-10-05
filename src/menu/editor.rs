// editor.rs
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{stdout, Write};
use std::path::PathBuf;
use crate::ansi_theme::AnsiTheme;
use crate::config::ThemeConfig;

pub struct TextEditor {
    content: Vec<String>,
    file_path: PathBuf,
    cursor_position: (usize, usize), // (column, row) in file coordinates
    viewport_offset: (usize, usize), // (horizontal scroll, vertical scroll)
    modified: bool,
    theme: ThemeConfig,
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

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> bool {
        // Only process key press events, ignore key release events
        if key_event.kind != KeyEventKind::Press {
            return true;
        }
        
        match (key_event.code, key_event.modifiers) {
            // Exit and Save
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                if self.modified {
                    let _ = self.save_file();
                }
                return false; // Exit editor
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if let Ok(()) = self.save_file() {
                    self.modified = false;
                }
            }
            
            // Cursor Movement
            (KeyCode::Up, _) => self.move_cursor_up(),
            (KeyCode::Down, _) => self.move_cursor_down(),
            (KeyCode::Left, _) => self.move_cursor_left(),
            (KeyCode::Right, _) => self.move_cursor_right(),
            (KeyCode::Home, _) => self.move_cursor_home(),
            (KeyCode::End, _) => self.move_cursor_end(),
            (KeyCode::PageUp, _) => self.move_cursor_page_up(),
            (KeyCode::PageDown, _) => self.move_cursor_page_down(),
            
            // Character input
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                self.insert_char(c);
            }
            (KeyCode::Backspace, _) => {
                self.delete_char_backspace();
            }
            (KeyCode::Delete, _) => {
                self.delete_char_delete();
            }
            (KeyCode::Enter, _) => {
                self.insert_newline();
            }
            
            _ => {} // Ignore other keys for now
        }
        true // Continue editor loop
    }

    // Cursor movement methods
    fn move_cursor_up(&mut self) {
        if self.cursor_position.1 > 0 {
            self.cursor_position.1 -= 1;
            // Adjust column to not exceed new line length
            let max_col = self.content[self.cursor_position.1].chars().count();
            if self.cursor_position.0 > max_col {
                self.cursor_position.0 = max_col;
            }
            self.adjust_viewport_to_cursor();
        }
    }

    fn move_cursor_down(&mut self) {
        if self.cursor_position.1 < self.content.len() - 1 {
            self.cursor_position.1 += 1;
            // Adjust column to not exceed new line length
            let max_col = self.content[self.cursor_position.1].chars().count();
            if self.cursor_position.0 > max_col {
                self.cursor_position.0 = max_col;
            }
            self.adjust_viewport_to_cursor();
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position.0 > 0 {
            self.cursor_position.0 -= 1;
        } else if self.cursor_position.1 > 0 {
            // Move to end of previous line
            self.cursor_position.1 -= 1;
            self.cursor_position.0 = self.content[self.cursor_position.1].chars().count();
        }
        self.adjust_viewport_to_cursor();
    }

    fn move_cursor_right(&mut self) {
        let current_line_len = self.content[self.cursor_position.1].chars().count();
        if self.cursor_position.0 < current_line_len {
            self.cursor_position.0 += 1;
        } else if self.cursor_position.1 < self.content.len() - 1 {
            // Move to start of next line
            self.cursor_position.1 += 1;
            self.cursor_position.0 = 0;
        }
        self.adjust_viewport_to_cursor();
    }

    fn move_cursor_home(&mut self) {
        self.cursor_position.0 = 0;
        self.adjust_viewport_to_cursor();
    }

    fn move_cursor_end(&mut self) {
        self.cursor_position.0 = self.content[self.cursor_position.1].chars().count();
        self.adjust_viewport_to_cursor();
    }

    fn move_cursor_page_up(&mut self) {
        let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
        let page_size = terminal_size.1 as usize - 2; // Account for status bar
        
        if self.cursor_position.1 >= page_size {
            self.cursor_position.1 -= page_size;
        } else {
            self.cursor_position.1 = 0;
        }
        
        // Adjust column for new line
        let max_col = self.content[self.cursor_position.1].chars().count();
        if self.cursor_position.0 > max_col {
            self.cursor_position.0 = max_col;
        }
        self.adjust_viewport_to_cursor();
    }

    fn move_cursor_page_down(&mut self) {
        let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
        let page_size = terminal_size.1 as usize - 2; // Account for status bar
        
        self.cursor_position.1 = std::cmp::min(
            self.cursor_position.1 + page_size,
            self.content.len() - 1
        );
        
        // Adjust column for new line
        let max_col = self.content[self.cursor_position.1].chars().count();
        if self.cursor_position.0 > max_col {
            self.cursor_position.0 = max_col;
        }
        self.adjust_viewport_to_cursor();
    }

    fn adjust_viewport_to_cursor(&mut self) {
        let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
        let term_width = terminal_size.0 as usize;
        let term_height = terminal_size.1 as usize - 1; // Reserve 1 line for status bar

        // Vertical adjustment
        if self.cursor_position.1 < self.viewport_offset.1 {
            self.viewport_offset.1 = self.cursor_position.1;
        } else if self.cursor_position.1 >= self.viewport_offset.1 + term_height {
            self.viewport_offset.1 = self.cursor_position.1 - term_height + 1;
        }

        // Horizontal adjustment
        if self.cursor_position.0 < self.viewport_offset.0 {
            self.viewport_offset.0 = self.cursor_position.0;
        } else if self.cursor_position.0 >= self.viewport_offset.0 + term_width {
            self.viewport_offset.0 = self.cursor_position.0 - term_width + 1;
        }
    }

    // Basic editing methods
    fn insert_char(&mut self, c: char) {
        let line = &mut self.content[self.cursor_position.1];
        let char_count = line.chars().count();
        
        if self.cursor_position.0 <= char_count {
            let mut chars: Vec<char> = line.chars().collect();
            chars.insert(self.cursor_position.0, c);
            *line = chars.into_iter().collect();
            self.cursor_position.0 += 1;
            self.modified = true;
            self.adjust_viewport_to_cursor();
        }
    }

    fn delete_char_backspace(&mut self) {
        if self.cursor_position.0 > 0 {
            let line = &mut self.content[self.cursor_position.1];
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_position.0 - 1);
            *line = chars.into_iter().collect();
            self.cursor_position.0 -= 1;
            self.modified = true;
            self.adjust_viewport_to_cursor();
        } else if self.cursor_position.1 > 0 {
            // Merge with previous line
            let current_line = self.content.remove(self.cursor_position.1);
            self.cursor_position.1 -= 1;
            let prev_line_len = self.content[self.cursor_position.1].chars().count();
            self.content[self.cursor_position.1].push_str(&current_line);
            self.cursor_position.0 = prev_line_len;
            self.modified = true;
            self.adjust_viewport_to_cursor();
        }
    }

    fn delete_char_delete(&mut self) {
        let line = &mut self.content[self.cursor_position.1];
        let char_count = line.chars().count();
        
        if self.cursor_position.0 < char_count {
            let mut chars: Vec<char> = line.chars().collect();
            chars.remove(self.cursor_position.0);
            *line = chars.into_iter().collect();
            self.modified = true;
            self.adjust_viewport_to_cursor();
        } else if self.cursor_position.1 < self.content.len() - 1 {
            // Merge with next line
            let next_line = self.content.remove(self.cursor_position.1 + 1);
            self.content[self.cursor_position.1].push_str(&next_line);
            self.modified = true;
            self.adjust_viewport_to_cursor();
        }
    }

    fn insert_newline(&mut self) {
        // Get the current line content without holding a mutable reference
        let current_line = self.content[self.cursor_position.1].clone();
        
        // Split the line at cursor position
        let prefix: String = current_line.chars().take(self.cursor_position.0).collect();
        let suffix: String = current_line.chars().skip(self.cursor_position.0).collect();
        
        // Update the current line and insert new line
        self.content[self.cursor_position.1] = prefix;
        self.content.insert(self.cursor_position.1 + 1, suffix);
        
        self.cursor_position.1 += 1;
        self.cursor_position.0 = 0;
        self.modified = true;
        self.adjust_viewport_to_cursor();
    }

    fn editor_loop(&mut self) -> Result<(), String> {
        loop {
            self.draw_interface()?;

            if let Event::Key(key_event) = event::read().map_err(|e| format!("Failed to read event: {}", e))? {
                if !self.handle_key_event(key_event) {
                    break;
                }
            }
        }
        Ok(())
    }

    fn draw_interface(&self) -> Result<(), String> {
        // Clear screen
        execute!(stdout(), crossterm::terminal::Clear(crossterm::terminal::ClearType::All))
            .map_err(|e| format!("Failed to clear screen: {}", e))?;

        // Draw content
        self.draw_content()?;

        // Draw status bar
        self.draw_status_bar()?;

        // Calculate screen position from file position and viewport
        let screen_x = (self.cursor_position.0 as isize - self.viewport_offset.0 as isize).max(0) as u16;
        let screen_y = (self.cursor_position.1 as isize - self.viewport_offset.1 as isize).max(0) as u16;

        // Position cursor
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(screen_x, screen_y)
        ).map_err(|e| format!("Failed to move cursor: {}", e))?;

        stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
        Ok(())
    }

    fn draw_status_bar(&self) -> Result<(), String> {
        let status = if self.modified {
            format!("{} [Modified] Line {}, Col {}", 
                self.file_path.display(), 
                self.cursor_position.1 + 1, 
                self.cursor_position.0 + 1)
        } else {
            format!("{} Line {}, Col {}", 
                self.file_path.display(), 
                self.cursor_position.1 + 1, 
                self.cursor_position.0 + 1)
        };

        let terminal_size = crossterm::terminal::size()
            .map_err(|e| format!("Failed to get terminal size: {}", e))?;
        
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(0, terminal_size.1 - 1)
        ).map_err(|e| format!("Failed to move cursor to status bar: {}", e))?;

        // Clear the status line first
        AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize), &self.theme);
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(0, terminal_size.1 - 1)
        ).map_err(|e| format!("Failed to move cursor to status bar: {}", e))?;

        AnsiTheme::print_themed(&status, &self.theme);
        Ok(())
    }

    fn draw_content(&self) -> Result<(), String> {
        let terminal_size = crossterm::terminal::size()
            .map_err(|e| format!("Failed to get terminal size: {}", e))?;
        let term_height = terminal_size.1 as usize - 1;

        for i in 0..term_height {
            let line_index = self.viewport_offset.1 + i;
            if line_index < self.content.len() {
                let line = &self.content[line_index];
                
                // Handle horizontal scrolling
                let display_line: String = if self.viewport_offset.0 < line.chars().count() {
                    line.chars()
                        .skip(self.viewport_offset.0)
                        .take(terminal_size.0 as usize)
                        .collect()
                } else {
                    String::new()
                };

                execute!(
                    stdout(),
                    crossterm::cursor::MoveTo(0, i as u16)
                ).map_err(|e| format!("Failed to move cursor for line {}: {}", i, e))?;
                
                AnsiTheme::print_themed(&display_line, &self.theme);
            } else {
                // Clear remaining lines
                execute!(
                    stdout(),
                    crossterm::cursor::MoveTo(0, i as u16)
                ).map_err(|e| format!("Failed to move cursor for line {}: {}", i, e))?;
                
                AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize), &self.theme);
            }
        }
        Ok(())
    }
}

// Public interface function
pub fn open_file_in_editor(file_path: PathBuf, theme: ThemeConfig) -> Result<(), String> {
    let mut editor = TextEditor::new(file_path, theme);
    editor.run_editor()
}