use super::core::TextEditor;

pub fn copy_selection(editor: &mut TextEditor) {
    if let Some((start, end)) = get_selection_range(editor) {
        let mut selected_text = String::new();
        
        if start.1 == end.1 {
            let line = &editor.content[start.1];
            if start.0 < line.chars().count() {
                selected_text = line.chars()
                    .skip(start.0)
                    .take(end.0 - start.0)
                    .collect();
            }
        } else {
            for row in start.1..=end.1 {
                if row < editor.content.len() {
                    let line = &editor.content[row];
                    if row == start.1 {
                        selected_text.push_str(&line.chars().skip(start.0).collect::<String>());
                    } else if row == end.1 {
                        let part: String = line.chars().take(end.0).collect();
                        selected_text.push_str(&part);
                    } else {
                        selected_text.push_str(line);
                    }
                    
                    if row < end.1 {
                        selected_text.push('\n');
                    }
                }
            }
        }
        
        editor.clipboard = selected_text;
        
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(&editor.clipboard);
        }
    }
}

pub fn paste_from_clipboard(editor: &mut TextEditor) {
    let text_to_paste = if let Ok(mut clipboard) = arboard::Clipboard::new() {
        clipboard.get_text().unwrap_or_else(|_| editor.clipboard.clone())
    } else {
        editor.clipboard.clone()
    };

    if !text_to_paste.is_empty() {
        if let Some((start, _)) = get_selection_range(editor) {
            delete_selection(editor);
            editor.cursor_position = start;
        }

        let lines: Vec<&str> = text_to_paste.split('\n').collect();
        
        if lines.len() == 1 {
            let line = &mut editor.content[editor.cursor_position.1];
            let mut chars: Vec<char> = line.chars().collect();
            
            for c in lines[0].chars() {
                chars.insert(editor.cursor_position.0, c);
                editor.cursor_position.0 += 1;
            }
            
            *line = chars.into_iter().collect();
        } else {
            let current_line = &editor.content[editor.cursor_position.1].clone();
            let prefix: String = current_line.chars().take(editor.cursor_position.0).collect();
            let suffix: String = current_line.chars().skip(editor.cursor_position.0).collect();
            
            editor.content[editor.cursor_position.1] = format!("{}{}", prefix, lines[0]);
            
            for i in 1..lines.len() - 1 {
                editor.content.insert(editor.cursor_position.1 + i, lines[i].to_string());
            }
            
            let last_line_index = editor.cursor_position.1 + lines.len() - 1;
            editor.content.insert(last_line_index, format!("{}{}", lines[lines.len() - 1], suffix));
            
            editor.cursor_position.1 = last_line_index;
            editor.cursor_position.0 = lines[lines.len() - 1].chars().count();
        }
        
        editor.modified = true;
        super::cursor::adjust_viewport_to_cursor_smooth(editor);
        editor.clear_selection();
    }
}

pub fn cut_selection(editor: &mut TextEditor) {
    if editor.has_selection() {
        copy_selection(editor);
        delete_selection(editor);
    }
}

// NEW: Replace selection with a single character
pub fn replace_selection_with_char(editor: &mut TextEditor, c: char) {
    if let Some((start, _)) = get_selection_range(editor) {
        delete_selection(editor);
        editor.cursor_position = start;
        super::cursor::insert_char(editor, c);
    }
}

pub fn delete_selection(editor: &mut TextEditor) {
    if let Some((start, end)) = get_selection_range(editor) {
        if start.1 == end.1 {
            let line = &mut editor.content[start.1];
            let mut chars: Vec<char> = line.chars().collect();
            
            for _ in start.0..end.0 {
                if start.0 < chars.len() {
                    chars.remove(start.0);
                }
            }
            
            *line = chars.into_iter().collect();
            editor.cursor_position = start;
        } else {
            let first_line_prefix: String = editor.content[start.1].chars().take(start.0).collect();
            let last_line_suffix: String = editor.content[end.1].chars().skip(end.0).collect();
            
            editor.content[start.1] = first_line_prefix;
            
            for _ in start.1 + 1..=end.1 {
                if start.1 + 1 < editor.content.len() {
                    editor.content.remove(start.1 + 1);
                }
            }
            
            editor.content[start.1].push_str(&last_line_suffix);
            editor.cursor_position = start;
        }
        
        editor.modified = true;
        editor.clear_selection();
        super::cursor::adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn start_or_extend_selection(editor: &mut TextEditor) {
    if editor.selection_start.is_none() {
        editor.selection_start = Some(editor.cursor_position);
    }
}

pub fn get_selection_range(editor: &TextEditor) -> Option<((usize, usize), (usize, usize))> {
    let start = editor.selection_start?;
    let end = editor.cursor_position;
    
    if start.1 < end.1 || (start.1 == end.1 && start.0 <= end.0) {
        Some((start, end))
    } else {
        Some((end, start))
    }
}