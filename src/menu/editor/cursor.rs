use super::core::TextEditor;

pub fn move_cursor_up(editor: &mut TextEditor) {
    if editor.cursor_position.1 > 0 {
        editor.desired_column = editor.cursor_position.0;
        editor.cursor_position.1 -= 1;
        
        let max_col = editor.content[editor.cursor_position.1].chars().count();
        editor.cursor_position.0 = if max_col > 0 {
            std::cmp::min(editor.desired_column, max_col)
        } else {
            0
        };
        adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn move_cursor_down(editor: &mut TextEditor) {
    if editor.cursor_position.1 < editor.content.len() - 1 {
        editor.desired_column = editor.cursor_position.0;
        editor.cursor_position.1 += 1;
        
        let max_col = editor.content[editor.cursor_position.1].chars().count();
        editor.cursor_position.0 = if max_col > 0 {
            std::cmp::min(editor.desired_column, max_col)
        } else {
            0
        };
        adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn move_cursor_left(editor: &mut TextEditor) {
    if editor.cursor_position.0 > 0 {
        editor.cursor_position.0 -= 1;
        editor.desired_column = editor.cursor_position.0;
    } else if editor.cursor_position.1 > 0 {
        editor.cursor_position.1 -= 1;
        editor.cursor_position.0 = editor.content[editor.cursor_position.1].chars().count();
        editor.desired_column = editor.cursor_position.0;
    }
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn move_cursor_right(editor: &mut TextEditor) {
    let current_line_len = editor.content[editor.cursor_position.1].chars().count();
    if editor.cursor_position.0 < current_line_len {
        editor.cursor_position.0 += 1;
        editor.desired_column = editor.cursor_position.0;
    } else if editor.cursor_position.1 < editor.content.len() - 1 {
        editor.cursor_position.1 += 1;
        editor.cursor_position.0 = 0;
        editor.desired_column = 0;
    }
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn move_cursor_home(editor: &mut TextEditor) {
    editor.cursor_position.0 = 0;
    editor.desired_column = 0;
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn move_cursor_end(editor: &mut TextEditor) {
    editor.cursor_position.0 = editor.content[editor.cursor_position.1].chars().count();
    editor.desired_column = editor.cursor_position.0;
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn move_cursor_page_up(editor: &mut TextEditor) {
    let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
    let page_size = terminal_size.1 as usize - 1;
    
    if editor.cursor_position.1 >= page_size {
        editor.cursor_position.1 -= page_size;
    } else {
        editor.cursor_position.1 = 0;
    }
    
    let max_col = editor.content[editor.cursor_position.1].chars().count();
    editor.cursor_position.0 = if max_col > 0 {
        std::cmp::min(editor.desired_column, max_col)
    } else {
        0
    };
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn move_cursor_page_down(editor: &mut TextEditor) {
    let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
    let page_size = terminal_size.1 as usize - 1;
    
    editor.cursor_position.1 = std::cmp::min(
        editor.cursor_position.1 + page_size,
        editor.content.len() - 1
    );
    
    let max_col = editor.content[editor.cursor_position.1].chars().count();
    editor.cursor_position.0 = if max_col > 0 {
        std::cmp::min(editor.desired_column, max_col)
    } else {
        0
    };
    adjust_viewport_to_cursor_smooth(editor);
}

pub fn adjust_viewport_to_cursor_smooth(editor: &mut TextEditor) {
    let terminal_size = crossterm::terminal::size().unwrap_or((80, 24));
    let term_width = terminal_size.0 as usize;
    let term_height = terminal_size.1 as usize - 1;

    if editor.cursor_position.1 < editor.viewport_offset.1 {
        editor.viewport_offset.1 = editor.cursor_position.1;
    } else if editor.cursor_position.1 >= editor.viewport_offset.1 + term_height {
        editor.viewport_offset.1 = editor.cursor_position.1 - term_height + 1;
    }

    if editor.cursor_position.0 < editor.viewport_offset.0 {
        editor.viewport_offset.0 = editor.cursor_position.0;
    } else if editor.cursor_position.0 >= editor.viewport_offset.0 + term_width {
        editor.viewport_offset.0 = editor.cursor_position.0 - term_width + 1;
    }

    let max_vertical_offset = if editor.content.len() > term_height {
        editor.content.len() - term_height
    } else {
        0
    };
    editor.viewport_offset.1 = std::cmp::min(editor.viewport_offset.1, max_vertical_offset);
}

pub fn insert_char(editor: &mut TextEditor, c: char) {
    let line = &mut editor.content[editor.cursor_position.1];
    let char_count = line.chars().count();
    
    if editor.cursor_position.0 <= char_count {
        let mut chars: Vec<char> = line.chars().collect();
        chars.insert(editor.cursor_position.0, c);
        *line = chars.into_iter().collect();
        editor.cursor_position.0 += 1;
        editor.desired_column = editor.cursor_position.0;
        editor.modified = true;
        adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn delete_char_backspace(editor: &mut TextEditor) {
    if editor.cursor_position.0 > 0 {
        let line = &mut editor.content[editor.cursor_position.1];
        let mut chars: Vec<char> = line.chars().collect();
        chars.remove(editor.cursor_position.0 - 1);
        *line = chars.into_iter().collect();
        editor.cursor_position.0 -= 1;
        editor.desired_column = editor.cursor_position.0;
        editor.modified = true;
        adjust_viewport_to_cursor_smooth(editor);
    } else if editor.cursor_position.1 > 0 {
        let current_line = editor.content.remove(editor.cursor_position.1);
        editor.cursor_position.1 -= 1;
        let prev_line_len = editor.content[editor.cursor_position.1].chars().count();
        editor.content[editor.cursor_position.1].push_str(&current_line);
        editor.cursor_position.0 = prev_line_len;
        editor.desired_column = editor.cursor_position.0;
        editor.modified = true;
        adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn delete_char_delete(editor: &mut TextEditor) {
    let line = &mut editor.content[editor.cursor_position.1];
    let char_count = line.chars().count();
    
    if editor.cursor_position.0 < char_count {
        let mut chars: Vec<char> = line.chars().collect();
        chars.remove(editor.cursor_position.0);
        *line = chars.into_iter().collect();
        editor.modified = true;
        adjust_viewport_to_cursor_smooth(editor);
    } else if editor.cursor_position.1 < editor.content.len() - 1 {
        let next_line = editor.content.remove(editor.cursor_position.1 + 1);
        editor.content[editor.cursor_position.1].push_str(&next_line);
        editor.modified = true;
        adjust_viewport_to_cursor_smooth(editor);
    }
}

pub fn insert_newline(editor: &mut TextEditor) {
    let current_line = editor.content[editor.cursor_position.1].clone();
    let prefix: String = current_line.chars().take(editor.cursor_position.0).collect();
    let suffix: String = current_line.chars().skip(editor.cursor_position.0).collect();
    
    editor.content[editor.cursor_position.1] = prefix;
    editor.content.insert(editor.cursor_position.1 + 1, suffix);
    
    editor.cursor_position.1 += 1;
    editor.cursor_position.0 = 0;
    editor.desired_column = 0;
    editor.modified = true;
    adjust_viewport_to_cursor_smooth(editor);
}
