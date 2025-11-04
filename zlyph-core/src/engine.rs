//! Core editor engine with platform-agnostic business logic

use crate::{BufferPosition, EditorAction, EditorState};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub struct EditorEngine {
    state: EditorState,
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
    last_edit_time: Option<Instant>,
}

const UNDO_CHUNK_DURATION: Duration = Duration::from_millis(500);

impl EditorEngine {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit_time: None,
        }
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut EditorState {
        &mut self.state
    }

    fn should_push_undo_state(&self) -> bool {
        if let Some(last_time) = self.last_edit_time {
            Instant::now().duration_since(last_time) > UNDO_CHUNK_DURATION
        } else {
            true
        }
    }

    fn push_undo_state(&mut self) {
        if !self.should_push_undo_state() {
            return;
        }
        self.undo_stack.push(self.state.clone_for_undo());
        self.redo_stack.clear();
    }

    fn mark_edit_time(&mut self) {
        self.last_edit_time = Some(Instant::now());
    }

    pub fn handle_action(&mut self, action: EditorAction) {
        match action {
            EditorAction::TypeCharacter(c) => self.type_character(c),
            EditorAction::TypeString(s) => self.type_string(&s),
            EditorAction::Backspace => self.backspace(),
            EditorAction::Delete => self.delete(),
            EditorAction::Newline => self.newline(),
            EditorAction::MoveLeft => self.move_left(),
            EditorAction::MoveRight => self.move_right(),
            EditorAction::MoveUp => self.move_up(),
            EditorAction::MoveDown => self.move_down(),
            EditorAction::MoveToBeginningOfLine => self.move_to_line_start(),
            EditorAction::MoveToEndOfLine => self.move_to_line_end(),
            EditorAction::MoveWordLeft => self.move_word_left(),
            EditorAction::MoveWordRight => self.move_word_right(),
            EditorAction::Undo => self.undo(),
            EditorAction::Redo => self.redo(),
            EditorAction::DeleteLine => self.delete_line(),
            EditorAction::DeleteToBeginningOfLine => self.delete_to_beginning_of_line(),
            EditorAction::DeleteToEndOfLine => self.delete_to_end_of_line(),
            EditorAction::MoveLineUp => self.move_line_up(),
            EditorAction::MoveLineDown => self.move_line_down(),
            EditorAction::Tab => self.tab(),
            EditorAction::Outdent => self.outdent(),
            EditorAction::SelectLeft => self.select_left(),
            EditorAction::SelectRight => self.select_right(),
            EditorAction::SelectUp => self.select_up(),
            EditorAction::SelectDown => self.select_down(),
            EditorAction::SelectWordLeft => self.select_word_left(),
            EditorAction::SelectWordRight => self.select_word_right(),
            EditorAction::SelectAll => self.select_all(),
            EditorAction::IncreaseFontSize => {
                self.state.font_size = (self.state.font_size + 2.0).min(72.0);
            }
            EditorAction::DecreaseFontSize => {
                self.state.font_size = (self.state.font_size - 2.0).max(8.0);
            }
            EditorAction::ResetFontSize => {
                self.state.font_size = 48.0;
            }
            EditorAction::Cut | EditorAction::Copy | EditorAction::Paste(_) => {
                // Clipboard operations need platform-specific handling
            }
            EditorAction::Quit => {
                // Handled by platform-specific code
            }
        }
    }

    fn selection_range(&self) -> Option<(BufferPosition, BufferPosition)> {
        self.state.selection_anchor.map(|anchor| {
            if anchor.row < self.state.cursor.row
                || (anchor.row == self.state.cursor.row
                    && anchor.column < self.state.cursor.column)
            {
                (anchor, self.state.cursor)
            } else {
                (self.state.cursor, anchor)
            }
        })
    }

    fn clear_selection(&mut self) {
        self.state.selection_anchor = None;
    }

    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            self.delete_range(start, end);
            self.state.cursor = start;
            self.clear_selection();
        }
    }

    fn delete_range(&mut self, start: BufferPosition, end: BufferPosition) {
        if start.row == end.row {
            let line = &mut self.state.lines[start.row];
            line.replace_range(start.column..end.column, "");
        } else {
            let first_part = self.state.lines[start.row][..start.column].to_string();
            let last_part = self.state.lines[end.row][end.column..].to_string();
            self.state.lines[start.row] = first_part + &last_part;
            self.state.lines.drain((start.row + 1)..=(end.row));
        }
    }

    fn type_character(&mut self, c: char) {
        self.push_undo_state();
        self.mark_edit_time();
        self.delete_selection();

        if c == '\n' {
            let line = self.state.lines[self.state.cursor.row].clone();
            let (before, after) = line.split_at(self.state.cursor.column);
            self.state.lines[self.state.cursor.row] = before.to_string();
            self.state.lines.insert(self.state.cursor.row + 1, after.to_string());
            self.state.cursor = BufferPosition::new(self.state.cursor.row + 1, 0);
        } else {
            self.state.lines[self.state.cursor.row].insert(self.state.cursor.column, c);
            self.state.cursor.column += c.len_utf8();
        }
    }

    fn type_string(&mut self, s: &str) {
        self.push_undo_state();
        self.mark_edit_time();
        self.delete_selection();

        for c in s.chars() {
            if c == '\n' {
                let line = self.state.lines[self.state.cursor.row].clone();
                let (before, after) = line.split_at(self.state.cursor.column);
                self.state.lines[self.state.cursor.row] = before.to_string();
                self.state.lines.insert(self.state.cursor.row + 1, after.to_string());
                self.state.cursor = BufferPosition::new(self.state.cursor.row + 1, 0);
            } else {
                self.state.lines[self.state.cursor.row].insert(self.state.cursor.column, c);
                self.state.cursor.column += c.len_utf8();
            }
        }
    }

    fn backspace(&mut self) {
        self.push_undo_state();
        self.mark_edit_time();

        if let Some((start, end)) = self.selection_range() {
            self.delete_range(start, end);
            self.state.cursor = start;
            self.clear_selection();
        } else if self.state.cursor.column > 0 {
            let line = &self.state.lines[self.state.cursor.row];
            let before = &line[..self.state.cursor.column];
            if let Some((last_char_start, _)) = before.char_indices().last() {
                self.state.lines[self.state.cursor.row].remove(last_char_start);
                self.state.cursor.column = last_char_start;
            }
        } else if self.state.cursor.row > 0 {
            let current_line = self.state.lines.remove(self.state.cursor.row);
            self.state.cursor.row -= 1;
            self.state.cursor.column = self.state.lines[self.state.cursor.row].len();
            self.state.lines[self.state.cursor.row].push_str(&current_line);
        }
    }

    fn delete(&mut self) {
        self.push_undo_state();
        self.mark_edit_time();

        if let Some((start, end)) = self.selection_range() {
            self.delete_range(start, end);
            self.state.cursor = start;
            self.clear_selection();
        } else {
            let line_len = self.state.lines[self.state.cursor.row].len();
            if self.state.cursor.column < line_len {
                self.state.lines[self.state.cursor.row].remove(self.state.cursor.column);
            } else if self.state.cursor.row + 1 < self.state.lines.len() {
                let next_line = self.state.lines.remove(self.state.cursor.row + 1);
                self.state.lines[self.state.cursor.row].push_str(&next_line);
            }
        }
    }

    fn detect_list_pattern(line: &str) -> Option<(String, usize, bool)> {
        let trimmed = line.trim_start();
        let indent_len = line.len() - trimmed.len();

        if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            return Some(("- [ ] ".to_string(), indent_len + 6, rest.is_empty()));
        }
        if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            return Some(("- [ ] ".to_string(), indent_len + 6, rest.is_empty()));
        }
        if let Some(rest) = trimmed.strip_prefix("- [X] ") {
            return Some(("- [ ] ".to_string(), indent_len + 6, rest.is_empty()));
        }
        if let Some(rest) = trimmed.strip_prefix("- ") {
            return Some(("- ".to_string(), indent_len + 2, rest.is_empty()));
        }
        if let Some(rest) = trimmed.strip_prefix("* ") {
            return Some(("* ".to_string(), indent_len + 2, rest.is_empty()));
        }
        if let Some(rest) = trimmed.strip_prefix("+ ") {
            return Some(("+ ".to_string(), indent_len + 2, rest.is_empty()));
        }

        if let Some(number_end) = trimmed.find(". ") {
            if let Ok(num) = trimmed[..number_end].parse::<usize>() {
                let rest = &trimmed[number_end + 2..];
                let next_num = num + 1;
                let pattern = format!("{}. ", next_num);
                return Some((pattern, indent_len + number_end + 2, rest.is_empty()));
            }
        }

        None
    }

    fn newline(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;
        self.delete_selection();

        let line = self.state.lines[self.state.cursor.row].clone();

        if let Some((pattern, pattern_len, is_empty)) = Self::detect_list_pattern(&line) {
            if is_empty {
                let before_pattern = &line[..line.len() - pattern_len];
                self.state.lines[self.state.cursor.row] = before_pattern.to_string();
                self.state.lines.insert(self.state.cursor.row + 1, String::new());
                self.state.cursor = BufferPosition::new(self.state.cursor.row + 1, 0);
            } else {
                let (before, after) = line.split_at(self.state.cursor.column);
                self.state.lines[self.state.cursor.row] = before.to_string();
                self.state.lines.insert(self.state.cursor.row + 1, pattern.clone() + after);
                self.state.cursor = BufferPosition::new(self.state.cursor.row + 1, pattern.len());
            }
        } else {
            let (before, after) = line.split_at(self.state.cursor.column);
            self.state.lines[self.state.cursor.row] = before.to_string();
            self.state.lines.insert(self.state.cursor.row + 1, after.to_string());
            self.state.cursor = BufferPosition::new(self.state.cursor.row + 1, 0);
        }
    }

    fn move_left(&mut self) {
        self.clear_selection();
        if self.state.cursor.column > 0 {
            let line = &self.state.lines[self.state.cursor.row];
            let before = &line[..self.state.cursor.column];
            if let Some(prev_char) = before.chars().last() {
                self.state.cursor.column -= prev_char.len_utf8();
            }
        } else if self.state.cursor.row > 0 {
            self.state.cursor.row -= 1;
            self.state.cursor.column = self.state.lines[self.state.cursor.row].len();
        }
    }

    fn move_right(&mut self) {
        self.clear_selection();
        let line_len = self.state.lines[self.state.cursor.row].len();
        if self.state.cursor.column < line_len {
            let after = &self.state.lines[self.state.cursor.row][self.state.cursor.column..];
            if let Some(next_char) = after.chars().next() {
                self.state.cursor.column += next_char.len_utf8();
            }
        } else if self.state.cursor.row + 1 < self.state.lines.len() {
            self.state.cursor.row += 1;
            self.state.cursor.column = 0;
        }
    }

    fn move_up(&mut self) {
        self.clear_selection();
        if self.state.cursor.row > 0 {
            self.state.cursor.row -= 1;
            let line_len = self.state.lines[self.state.cursor.row].len();
            self.state.cursor.column = self.state.cursor.column.min(line_len);
        }
    }

    fn move_down(&mut self) {
        self.clear_selection();
        if self.state.cursor.row + 1 < self.state.lines.len() {
            self.state.cursor.row += 1;
            let line_len = self.state.lines[self.state.cursor.row].len();
            self.state.cursor.column = self.state.cursor.column.min(line_len);
        }
    }

    fn move_to_line_start(&mut self) {
        self.clear_selection();
        self.state.cursor.column = 0;
    }

    fn move_to_line_end(&mut self) {
        self.clear_selection();
        self.state.cursor.column = self.state.lines[self.state.cursor.row].len();
    }

    fn move_word_left(&mut self) {
        self.clear_selection();
        // Simplified word movement
        self.move_left();
    }

    fn move_word_right(&mut self) {
        self.clear_selection();
        // Simplified word movement
        self.move_right();
    }

    fn undo(&mut self) {
        if let Some(prev_state) = self.undo_stack.pop() {
            self.redo_stack.push(self.state.clone_for_undo());
            self.state = prev_state;
            self.last_edit_time = None;
        }
    }

    fn redo(&mut self) {
        if let Some(next_state) = self.redo_stack.pop() {
            self.undo_stack.push(self.state.clone_for_undo());
            self.state = next_state;
            self.last_edit_time = None;
        }
    }

    fn delete_line(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;

        if self.state.lines.len() == 1 {
            self.state.lines[0].clear();
            self.state.cursor = BufferPosition::zero();
        } else if self.state.cursor.row < self.state.lines.len() - 1 {
            self.state.lines.remove(self.state.cursor.row);
            self.state.cursor.column = 0;
        } else {
            self.state.lines.remove(self.state.cursor.row);
            self.state.cursor.row -= 1;
            self.state.cursor.column = 0;
        }
        self.clear_selection();
    }

    fn delete_to_beginning_of_line(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;
        self.state.lines[self.state.cursor.row].replace_range(..self.state.cursor.column, "");
        self.state.cursor.column = 0;
    }

    fn delete_to_end_of_line(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;
        self.state.lines[self.state.cursor.row].replace_range(self.state.cursor.column.., "");
    }

    fn move_line_up(&mut self) {
        if self.state.cursor.row == 0 {
            return;
        }
        self.push_undo_state();
        self.last_edit_time = None;
        self.state.lines.swap(self.state.cursor.row, self.state.cursor.row - 1);
        self.state.cursor.row -= 1;
    }

    fn move_line_down(&mut self) {
        if self.state.cursor.row + 1 >= self.state.lines.len() {
            return;
        }
        self.push_undo_state();
        self.last_edit_time = None;
        self.state.lines.swap(self.state.cursor.row, self.state.cursor.row + 1);
        self.state.cursor.row += 1;
    }

    fn tab(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;

        if let Some((start, end)) = self.selection_range() {
            for row in start.row..=end.row {
                self.state.lines[row].insert_str(0, "    ");
            }
            self.state.selection_anchor = Some(BufferPosition::new(start.row, start.column + 4));
            self.state.cursor = BufferPosition::new(end.row, end.column + 4);
        } else {
            self.state.lines[self.state.cursor.row].insert_str(self.state.cursor.column, "    ");
            self.state.cursor.column += 4;
        }
    }

    fn outdent(&mut self) {
        self.push_undo_state();
        self.last_edit_time = None;

        if let Some((start, end)) = self.selection_range() {
            for row in start.row..=end.row {
                let spaces_to_remove = self.state.lines[row]
                    .chars()
                    .take(4)
                    .take_while(|&c| c == ' ')
                    .count();
                if spaces_to_remove > 0 {
                    self.state.lines[row].replace_range(..spaces_to_remove, "");
                }
            }
            let new_start_col = start.column.saturating_sub(4);
            let new_end_col = end.column.saturating_sub(4);
            self.state.selection_anchor = Some(BufferPosition::new(start.row, new_start_col));
            self.state.cursor = BufferPosition::new(end.row, new_end_col);
        } else {
            let spaces_to_remove = self.state.lines[self.state.cursor.row]
                .chars()
                .take(4)
                .take_while(|&c| c == ' ')
                .count();
            if spaces_to_remove > 0 {
                self.state.lines[self.state.cursor.row].replace_range(..spaces_to_remove, "");
                self.state.cursor.column = self.state.cursor.column.saturating_sub(spaces_to_remove);
            }
        }
    }

    fn select_left(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        if self.state.cursor.column > 0 {
            let line = &self.state.lines[self.state.cursor.row];
            let before = &line[..self.state.cursor.column];
            if let Some(prev_char) = before.chars().last() {
                self.state.cursor.column -= prev_char.len_utf8();
            }
        } else if self.state.cursor.row > 0 {
            self.state.cursor.row -= 1;
            self.state.cursor.column = self.state.lines[self.state.cursor.row].len();
        }
    }

    fn select_right(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        let line_len = self.state.lines[self.state.cursor.row].len();
        if self.state.cursor.column < line_len {
            let after = &self.state.lines[self.state.cursor.row][self.state.cursor.column..];
            if let Some(next_char) = after.chars().next() {
                self.state.cursor.column += next_char.len_utf8();
            }
        } else if self.state.cursor.row + 1 < self.state.lines.len() {
            self.state.cursor.row += 1;
            self.state.cursor.column = 0;
        }
    }

    fn select_up(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        if self.state.cursor.row > 0 {
            self.state.cursor.row -= 1;
            let line_len = self.state.lines[self.state.cursor.row].len();
            self.state.cursor.column = self.state.cursor.column.min(line_len);
        }
    }

    fn select_down(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        if self.state.cursor.row + 1 < self.state.lines.len() {
            self.state.cursor.row += 1;
            let line_len = self.state.lines[self.state.cursor.row].len();
            self.state.cursor.column = self.state.cursor.column.min(line_len);
        }
    }

    fn select_word_left(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        self.move_word_left();
    }

    fn select_word_right(&mut self) {
        if self.state.selection_anchor.is_none() {
            self.state.selection_anchor = Some(self.state.cursor);
        }
        self.move_word_right();
    }

    fn select_all(&mut self) {
        self.state.selection_anchor = Some(BufferPosition::zero());
        let last_row = self.state.lines.len().saturating_sub(1);
        let last_col = self.state.lines[last_row].len();
        self.state.cursor = BufferPosition::new(last_row, last_col);
    }

    /// Load editor state from a file
    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let content = fs::read_to_string(path)?;
        self.state.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.state.cursor = BufferPosition::zero();
        self.state.selection_anchor = None;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.last_edit_time = None;
        Ok(())
    }

    /// Save editor state to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let content = self.state.lines.join("\n");
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    /// Get default config file path
    pub fn default_file_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config").join("zlyph").join("default.txt")
    }
}

impl Default for EditorEngine {
    fn default() -> Self {
        Self::new()
    }
}
