use std::io::{stdout, Write};
use crossterm::{execute, terminal};
use crate::ansi_theme::AnsiTheme;
use super::core::TextEditor;

pub fn draw_interface(editor: &TextEditor) -> Result<(), String> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::Purge))
        .map_err(|e| format!("Failed to clear screen: {}", e))?;

    draw_content(editor)?;

    if editor.exit_requested {
        draw_exit_confirmation(editor)?;
    } else {
        draw_status_bar(editor)?;
    }

    let screen_x = (editor.cursor_position.0 as isize - editor.viewport_offset.0 as isize).max(0) as u16;
    let screen_y = (editor.cursor_position.1 as isize - editor.viewport_offset.1 as isize).max(0) as u16;

    if !editor.exit_requested {
        execute!(
            stdout(),
            crossterm::cursor::MoveTo(screen_x, screen_y)
        ).map_err(|e| format!("Failed to move cursor: {}", e))?;
    }

    stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;
    Ok(())
}

pub fn draw_exit_confirmation(editor: &TextEditor) -> Result<(), String> {
    let terminal_size = crossterm::terminal::size()
        .map_err(|e| format!("Failed to get terminal size: {}", e))?;
    
    let message = "You have unsaved changes. Save before exiting? (y/n/c)";
    let y_pos = terminal_size.1 / 2;
    
    execute!(stdout(), crossterm::cursor::MoveTo(0, y_pos))
        .map_err(|e| format!("Failed to move cursor for confirmation: {}", e))?;
    
    AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize), &editor.theme);
    execute!(stdout(), crossterm::cursor::MoveTo(0, y_pos))
        .map_err(|e| format!("Failed to move cursor for confirmation: {}", e))?;
    
    let x_pos = (terminal_size.0.saturating_sub(message.len() as u16)) / 2;
    execute!(stdout(), crossterm::cursor::MoveTo(x_pos, y_pos))
        .map_err(|e| format!("Failed to move cursor for confirmation: {}", e))?;
    
    AnsiTheme::print_themed(message, &editor.theme);
    Ok(())
}

pub fn draw_status_bar(editor: &TextEditor) -> Result<(), String> {
    let selection_info = if editor.has_selection() {
        if let Some((start, end)) = super::clipboard::get_selection_range(editor) {
            format!(" | Selection: {}:{} to {}:{}", 
                start.1 + 1, start.0 + 1, end.1 + 1, end.0 + 1)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let status = if editor.modified {
        format!("{} [Modified] Line {}, Col {}{}", 
            editor.file_path.display(), 
            editor.cursor_position.1 + 1, 
            editor.cursor_position.0 + 1,
            selection_info)
    } else {
        format!("{} Line {}, Col {}{}", 
            editor.file_path.display(), 
            editor.cursor_position.1 + 1, 
            editor.cursor_position.0 + 1,
            selection_info)
    };

    let terminal_size = crossterm::terminal::size()
        .map_err(|e| format!("Failed to get terminal size: {}", e))?;
    
    execute!(stdout(), crossterm::cursor::MoveTo(0, terminal_size.1 - 1))
        .map_err(|e| format!("Failed to move cursor to status bar: {}", e))?;

    AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize), &editor.theme);
    execute!(stdout(), crossterm::cursor::MoveTo(0, terminal_size.1 - 1))
        .map_err(|e| format!("Failed to move cursor to status bar: {}", e))?;

    AnsiTheme::print_themed(&status, &editor.theme);
    Ok(())
}

pub fn draw_content(editor: &TextEditor) -> Result<(), String> {
    let terminal_size = crossterm::terminal::size()
        .map_err(|e| format!("Failed to get terminal size: {}", e))?;
    let term_height = terminal_size.1 as usize - 1;

    let selection_range = super::clipboard::get_selection_range(editor);

    for i in 0..term_height {
        let line_index = editor.viewport_offset.1 + i;
        
        execute!(stdout(), crossterm::cursor::MoveTo(0, i as u16))
            .map_err(|e| format!("Failed to move cursor for line {}: {}", i, e))?;

        if line_index < editor.content.len() {
            let line = &editor.content[line_index];
            let line_chars: Vec<char> = line.chars().collect();
            let line_len = line_chars.len();
            
            let mut display_pos = 0;
            let mut current_pos = editor.viewport_offset.0;
            
            while display_pos < terminal_size.0 as usize && current_pos < line_len {
                let is_selected = if let Some((start, end)) = selection_range {
                    line_index >= start.1 && line_index <= end.1 && 
                    current_pos >= if line_index == start.1 { start.0 } else { 0 } &&
                    current_pos < if line_index == end.1 { end.0 } else { line_len }
                } else {
                    false
                };
                
                if is_selected {
                    execute!(stdout(), crossterm::style::SetAttribute(crossterm::style::Attribute::Reverse))
                        .map_err(|e| format!("Failed to set reverse attribute: {}", e))?;
                }
                
                let c = line_chars[current_pos];
                AnsiTheme::print_themed(&c.to_string(), &editor.theme);
                
                if is_selected {
                    execute!(stdout(), crossterm::style::SetAttribute(crossterm::style::Attribute::NoReverse))
                        .map_err(|e| format!("Failed to reset attribute: {}", e))?;
                }
                
                display_pos += 1;
                current_pos += 1;
            }
            
            if display_pos < terminal_size.0 as usize {
                AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize - display_pos), &editor.theme);
            }
        } else {
            AnsiTheme::print_themed(&" ".repeat(terminal_size.0 as usize), &editor.theme);
        }
    }
    Ok(())
}