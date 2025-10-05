use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};
use super::core::TextEditor;
use super::clipboard;
use super::cursor;

pub fn handle_key_event(editor: &mut TextEditor, key_event: crossterm::event::KeyEvent) -> bool {
    if key_event.kind != KeyEventKind::Press {
        return true;
    }
    
    editor.needs_redraw = true;
    
    match (key_event.code, key_event.modifiers) {
        (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
            if editor.modified {
                editor.exit_requested = true;
            } else {
                return false;
            }
        }
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
            if let Ok(()) = editor.save_file() {
                editor.modified = false;
                editor.exit_requested = false;
            }
        }
        (KeyCode::Char('c'), KeyModifiers::CONTROL) if !editor.exit_requested => {
            clipboard::copy_selection(editor);
        }
        (KeyCode::Char('v'), KeyModifiers::CONTROL) if !editor.exit_requested => {
            clipboard::paste_from_clipboard(editor);
        }
        (KeyCode::Char('x'), KeyModifiers::CONTROL) if !editor.exit_requested => {
            clipboard::cut_selection(editor);
        }
        (KeyCode::Char('y'), KeyModifiers::NONE) if editor.exit_requested => {
            let _ = editor.save_file();
            return false;
        }
        (KeyCode::Char('n'), KeyModifiers::NONE) if editor.exit_requested => {
            return false;
        }
        (KeyCode::Char('c'), KeyModifiers::NONE) if editor.exit_requested => {
            editor.exit_requested = false;
        }
        (KeyCode::Esc, _) if editor.exit_requested => {
            editor.exit_requested = false;
        }
        (KeyCode::Up, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_up(editor);
        }
        (KeyCode::Down, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_down(editor);
        }
        (KeyCode::Left, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_left(editor);
        }
        (KeyCode::Right, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_right(editor);
        }
        (KeyCode::Home, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_home(editor);
        }
        (KeyCode::End, KeyModifiers::SHIFT) if !editor.exit_requested => {
            clipboard::start_or_extend_selection(editor);
            cursor::move_cursor_end(editor);
        }
        (KeyCode::Up, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_up(editor);
        }
        (KeyCode::Down, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_down(editor);
        }
        (KeyCode::Left, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_left(editor);
        }
        (KeyCode::Right, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_right(editor);
        }
        (KeyCode::Home, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_home(editor);
        }
        (KeyCode::End, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_end(editor);
        }
        (KeyCode::PageUp, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_page_up(editor);
        }
        (KeyCode::PageDown, _) if !editor.exit_requested => {
            editor.clear_selection();
            cursor::move_cursor_page_down(editor);
        }
        
        // NEW: Character input with selection replacement
        (KeyCode::Char(c), KeyModifiers::NONE) if !editor.exit_requested => {
            if editor.has_selection() {
                // Replace selection with typed character
                clipboard::replace_selection_with_char(editor, c);
            } else {
                cursor::insert_char(editor, c);
            }
        }
        (KeyCode::Char(c), KeyModifiers::SHIFT) if !editor.exit_requested => {
            if editor.has_selection() {
                // Replace selection with typed character
                clipboard::replace_selection_with_char(editor, c);
            } else {
                cursor::insert_char(editor, c);
            }
        }
        
        // NEW: Backspace with selection deletion
        (KeyCode::Backspace, _) if !editor.exit_requested => {
            if editor.has_selection() {
                clipboard::delete_selection(editor);
            } else {
                cursor::delete_char_backspace(editor);
            }
        }
        
        // NEW: Delete with selection deletion  
        (KeyCode::Delete, _) if !editor.exit_requested => {
            if editor.has_selection() {
                clipboard::delete_selection(editor);
            } else {
                cursor::delete_char_delete(editor);
            }
        }
        
        (KeyCode::Enter, _) if !editor.exit_requested => {
            if editor.has_selection() {
                // Replace selection with newline
                clipboard::delete_selection(editor);
            }
            cursor::insert_newline(editor);
        }
        _ => {
            editor.needs_redraw = false;
        }
    }
    true
}