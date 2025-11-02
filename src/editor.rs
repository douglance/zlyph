use crate::actions::*;
use crate::text_buffer::{BufferPosition, TextBuffer, WrapType};
use crate::theme::AtomOneDark;
use gpui::prelude::*;
use gpui::*;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct EditorState {
    buffer_content: String,
    cursor: BufferPosition,
    selection_anchor: Option<BufferPosition>,
}

pub struct TextEditor {
    buffer: TextBuffer,
    font_size: f32,
    cursor: BufferPosition,
    selection_anchor: Option<BufferPosition>,
    focus_handle: FocusHandle,
    theme: AtomOneDark,
    is_dragging: bool,
    undo_stack: Vec<EditorState>,
    redo_stack: Vec<EditorState>,
    last_edit_time: Option<Instant>,
    last_click_time: Option<Instant>,
    last_click_position: Option<BufferPosition>,
}

impl TextEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            buffer: TextBuffer::new(),
            font_size: 48.0,
            cursor: BufferPosition::zero(),
            selection_anchor: None,
            focus_handle: cx.focus_handle(),
            theme: AtomOneDark::default(),
            is_dragging: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            last_edit_time: None,
            last_click_time: None,
            last_click_position: None,
        }
    }

    fn should_push_undo_state(&self) -> bool {
        const UNDO_CHUNK_DURATION: Duration = Duration::from_millis(500);

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

        let state = EditorState {
            buffer_content: self.buffer.to_string(),
            cursor: self.cursor,
            selection_anchor: self.selection_anchor,
        };
        self.undo_stack.push(state);
        self.redo_stack.clear();
    }

    fn mark_edit_time(&mut self) {
        self.last_edit_time = Some(Instant::now());
    }

    fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prev_state) = self.undo_stack.pop() {
            let current_state = EditorState {
                buffer_content: self.buffer.to_string(),
                cursor: self.cursor,
                selection_anchor: self.selection_anchor,
            };
            self.redo_stack.push(current_state);

            self.buffer = TextBuffer::from_string(prev_state.buffer_content);
            self.cursor = prev_state.cursor;
            self.selection_anchor = prev_state.selection_anchor;
            self.last_edit_time = None;

            cx.notify();
        }
    }

    fn redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(next_state) = self.redo_stack.pop() {
            let current_state = EditorState {
                buffer_content: self.buffer.to_string(),
                cursor: self.cursor,
                selection_anchor: self.selection_anchor,
            };
            self.undo_stack.push(current_state);

            self.buffer = TextBuffer::from_string(next_state.buffer_content);
            self.cursor = next_state.cursor;
            self.selection_anchor = next_state.selection_anchor;
            self.last_edit_time = None;

            cx.notify();
        }
    }

    fn selection_range(&self) -> Option<(BufferPosition, BufferPosition)> {
        self.selection_anchor.map(|anchor| {
            if anchor.row < self.cursor.row || (anchor.row == self.cursor.row && anchor.column < self.cursor.column) {
                (anchor, self.cursor)
            } else {
                (self.cursor, anchor)
            }
        })
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn increase_font_size(&mut self, _: &IncreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size + 2.0).min(72.0);
        self.buffer.invalidate_all_layouts();
        cx.notify();
    }

    fn decrease_font_size(&mut self, _: &DecreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size - 2.0).max(8.0);
        self.buffer.invalidate_all_layouts();
        cx.notify();
    }

    fn reset_font_size(&mut self, _: &ResetFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = 48.0;
        self.buffer.invalidate_all_layouts();
        cx.notify();
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

    fn handle_newline(&mut self, _: &Newline, _window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;
        if let Some((start, end)) = self.selection_range() {
            self.buffer.delete_range(start, end);
            self.cursor = start;
            self.clear_selection();
        }

        if let Some(current_line) = self.buffer.line(self.cursor.row) {
            if let Some((pattern, pattern_len, is_empty)) = Self::detect_list_pattern(current_line) {
                if is_empty {
                    let line_start = BufferPosition::new(self.cursor.row, 0);
                    let line_end = BufferPosition::new(self.cursor.row, pattern_len);
                    self.buffer.delete_range(line_start, line_end);
                    self.cursor = line_start;
                    self.buffer.insert_char(self.cursor, '\n');
                    self.cursor = BufferPosition::new(self.cursor.row + 1, 0);
                } else {
                    self.buffer.insert_char(self.cursor, '\n');
                    self.cursor = BufferPosition::new(self.cursor.row + 1, 0);
                    self.buffer.insert_str(self.cursor, &pattern);
                    self.cursor.column += pattern.len();
                }
                cx.notify();
                return;
            }
        }

        self.buffer.insert_char(self.cursor, '\n');
        self.cursor = BufferPosition::new(self.cursor.row + 1, 0);
        cx.notify();
    }

    fn handle_backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.mark_edit_time();
        if let Some((start, end)) = self.selection_range() {
            self.buffer.delete_range(start, end);
            self.cursor = start;
            self.clear_selection();
        } else if self.buffer.backspace(self.cursor) {
            if self.cursor.column > 0 {
                self.cursor.column -= 1;
                let line = self.buffer.line(self.cursor.row).unwrap_or("");
                while self.cursor.column > 0 && !line.is_char_boundary(self.cursor.column) {
                    self.cursor.column -= 1;
                }
            } else if self.cursor.row > 0 {
                let prev_line_len = self.buffer.line_len(self.cursor.row - 1);
                self.cursor = BufferPosition::new(self.cursor.row - 1, prev_line_len);
            }
        }
        cx.notify();
    }

    fn handle_delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.mark_edit_time();
        if let Some((start, end)) = self.selection_range() {
            self.buffer.delete_range(start, end);
            self.cursor = start;
            self.clear_selection();
        } else {
            self.buffer.delete_char(self.cursor);
        }
        cx.notify();
    }

    fn delete_to_beginning_of_line(&mut self, _: &DeleteToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;
        let start = BufferPosition::new(self.cursor.row, 0);
        self.buffer.delete_range(start, self.cursor);
        self.cursor = start;
        cx.notify();
    }

    fn delete_to_end_of_line(&mut self, _: &DeleteToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;
        let line_len = self.buffer.line_len(self.cursor.row);
        let end = BufferPosition::new(self.cursor.row, line_len);
        self.buffer.delete_range(self.cursor, end);
        cx.notify();
    }

    fn move_to_beginning_of_line(&mut self, _: &MoveToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor = self.buffer.visual_line_start(self.cursor);
        cx.notify();
    }

    fn move_to_end_of_line(&mut self, _: &MoveToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor = self.buffer.visual_line_end(self.cursor);
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
            if let Some(line) = self.buffer.line(self.cursor.row) {
                while self.cursor.column > 0 && !line.is_char_boundary(self.cursor.column) {
                    self.cursor.column -= 1;
                }
            }
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.column = self.buffer.line_len(self.cursor.row);
        }
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        let line_len = self.buffer.line_len(self.cursor.row);
        if self.cursor.column < line_len {
            self.cursor.column += 1;
            if let Some(line) = self.buffer.line(self.cursor.row) {
                while self.cursor.column < line.len() && !line.is_char_boundary(self.cursor.column) {
                    self.cursor.column += 1;
                }
            }
        } else if self.cursor.row + 1 < self.buffer.line_count() {
            self.cursor.row += 1;
            self.cursor.column = 0;
        }
        cx.notify();
    }

    fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor = self.buffer.move_visual_up(self.cursor);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor = self.buffer.move_visual_down(self.cursor);
        cx.notify();
    }

    fn move_word_left(&mut self, _: &MoveWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if let Some(line) = self.buffer.line(self.cursor.row) {
            if self.cursor.column == 0 {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                    self.cursor.column = self.buffer.line_len(self.cursor.row);
                }
                cx.notify();
                return;
            }

            let chars: Vec<char> = line.chars().collect();
            let mut char_pos = line[..self.cursor.column].chars().count();

            if char_pos == 0 {
                cx.notify();
                return;
            }

            char_pos -= 1;
            while char_pos > 0 && chars[char_pos].is_whitespace() {
                char_pos -= 1;
            }

            if char_pos > 0 {
                let is_alphanumeric = chars[char_pos].is_alphanumeric() || chars[char_pos] == '_';
                while char_pos > 0 {
                    let prev_char = chars[char_pos - 1];
                    let prev_is_alphanumeric = prev_char.is_alphanumeric() || prev_char == '_';
                    if is_alphanumeric != prev_is_alphanumeric || prev_char.is_whitespace() {
                        break;
                    }
                    char_pos -= 1;
                }
            }

            let byte_pos: usize = chars[..char_pos].iter().map(|c| c.len_utf8()).sum();
            self.cursor.column = byte_pos;
        }
        cx.notify();
    }

    fn move_word_right(&mut self, _: &MoveWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if let Some(line) = self.buffer.line(self.cursor.row) {
            if self.cursor.column >= line.len() {
                if self.cursor.row + 1 < self.buffer.line_count() {
                    self.cursor.row += 1;
                    self.cursor.column = 0;
                }
                cx.notify();
                return;
            }

            let after = &line[self.cursor.column..];
            let chars: Vec<char> = after.chars().collect();
            let mut char_pos = 0;

            if chars.is_empty() {
                cx.notify();
                return;
            }

            while char_pos < chars.len() && chars[char_pos].is_whitespace() {
                char_pos += 1;
            }

            if char_pos < chars.len() {
                let is_alphanumeric = chars[char_pos].is_alphanumeric() || chars[char_pos] == '_';
                while char_pos < chars.len() {
                    let curr_char = chars[char_pos];
                    let curr_is_alphanumeric = curr_char.is_alphanumeric() || curr_char == '_';
                    if is_alphanumeric != curr_is_alphanumeric || curr_char.is_whitespace() {
                        break;
                    }
                    char_pos += 1;
                }
            }

            let byte_offset: usize = chars[..char_pos].iter().map(|c| c.len_utf8()).sum();
            self.cursor.column += byte_offset;
        }
        cx.notify();
    }

    fn move_line_up(&mut self, _: &MoveLineUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor.row == 0 {
            return;
        }

        self.push_undo_state();
        self.last_edit_time = None;

        let current_row = self.cursor.row;
        let current_line = self.buffer.line(current_row).unwrap_or("").to_string();
        let prev_line = self.buffer.line(current_row - 1).unwrap_or("").to_string();

        let start_prev = BufferPosition::new(current_row - 1, 0);
        let end_prev = BufferPosition::new(current_row - 1, prev_line.len());
        self.buffer.delete_range(start_prev, end_prev);
        self.buffer.delete_char(start_prev);

        let start_current = BufferPosition::new(current_row - 1, 0);
        let end_current = BufferPosition::new(current_row - 1, current_line.len());
        self.buffer.delete_range(start_current, end_current);

        self.buffer.insert_str(start_current, &current_line);
        self.buffer.insert_char(BufferPosition::new(current_row - 1, current_line.len()), '\n');
        self.buffer.insert_str(BufferPosition::new(current_row, 0), &prev_line);

        self.cursor.row -= 1;
        cx.notify();
    }

    fn move_line_down(&mut self, _: &MoveLineDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.cursor.row + 1 >= self.buffer.line_count() {
            return;
        }

        self.push_undo_state();
        self.last_edit_time = None;

        let current_row = self.cursor.row;
        let current_line = self.buffer.line(current_row).unwrap_or("").to_string();
        let next_line = self.buffer.line(current_row + 1).unwrap_or("").to_string();

        let start_current = BufferPosition::new(current_row, 0);
        let end_current = BufferPosition::new(current_row, current_line.len());
        self.buffer.delete_range(start_current, end_current);
        self.buffer.delete_char(start_current);

        let start_next = BufferPosition::new(current_row, 0);
        let end_next = BufferPosition::new(current_row, next_line.len());
        self.buffer.delete_range(start_next, end_next);

        self.buffer.insert_str(start_next, &next_line);
        self.buffer.insert_char(BufferPosition::new(current_row, next_line.len()), '\n');
        self.buffer.insert_str(BufferPosition::new(current_row + 1, 0), &current_line);

        self.cursor.row += 1;
        cx.notify();
    }

    fn delete_line(&mut self, _: &DeleteLine, _: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;

        let current_row = self.cursor.row;
        let line_len = self.buffer.line_len(current_row);

        let start = BufferPosition::new(current_row, 0);
        let end = BufferPosition::new(current_row, line_len);
        self.buffer.delete_range(start, end);

        if current_row < self.buffer.line_count() {
            self.buffer.delete_char(start);
        } else if current_row > 0 {
            self.buffer.delete_char(BufferPosition::new(current_row - 1, self.buffer.line_len(current_row - 1)));
            self.cursor.row -= 1;
        }

        self.cursor.column = 0;
        self.clear_selection();
        cx.notify();
    }

    fn handle_tab(&mut self, _: &Tab, _: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;

        if let Some((start, end)) = self.selection_range() {
            for row in start.row..=end.row {
                self.buffer.insert_str(BufferPosition::new(row, 0), "    ");
            }
            self.selection_anchor = Some(BufferPosition::new(start.row, start.column + 4));
            self.cursor = BufferPosition::new(end.row, end.column + 4);
        } else {
            self.buffer.insert_str(self.cursor, "    ");
            self.cursor.column += 4;
        }

        cx.notify();
    }

    fn handle_outdent(&mut self, _: &Outdent, _: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_state();
        self.last_edit_time = None;

        if let Some((start, end)) = self.selection_range() {
            for row in start.row..=end.row {
                if let Some(line) = self.buffer.line(row) {
                    let spaces_to_remove = line.chars().take(4).take_while(|&c| c == ' ').count();
                    if spaces_to_remove > 0 {
                        self.buffer.delete_range(
                            BufferPosition::new(row, 0),
                            BufferPosition::new(row, spaces_to_remove),
                        );
                    }
                }
            }
            let new_start_col = start.column.saturating_sub(4);
            let new_end_col = end.column.saturating_sub(4);
            self.selection_anchor = Some(BufferPosition::new(start.row, new_start_col));
            self.cursor = BufferPosition::new(end.row, new_end_col);
        } else {
            if let Some(line) = self.buffer.line(self.cursor.row) {
                let spaces_to_remove = line.chars().take(4).take_while(|&c| c == ' ').count();
                if spaces_to_remove > 0 {
                    self.buffer.delete_range(
                        BufferPosition::new(self.cursor.row, 0),
                        BufferPosition::new(self.cursor.row, spaces_to_remove),
                    );
                    self.cursor.column = self.cursor.column.saturating_sub(spaces_to_remove);
                }
            }
        }

        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        if self.cursor.column > 0 {
            self.cursor.column -= 1;
            if let Some(line) = self.buffer.line(self.cursor.row) {
                while self.cursor.column > 0 && !line.is_char_boundary(self.cursor.column) {
                    self.cursor.column -= 1;
                }
            }
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.column = self.buffer.line_len(self.cursor.row);
        }
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        let line_len = self.buffer.line_len(self.cursor.row);
        if self.cursor.column < line_len {
            self.cursor.column += 1;
            if let Some(line) = self.buffer.line(self.cursor.row) {
                while self.cursor.column < line.len() && !line.is_char_boundary(self.cursor.column) {
                    self.cursor.column += 1;
                }
            }
        } else if self.cursor.row + 1 < self.buffer.line_count() {
            self.cursor.row += 1;
            self.cursor.column = 0;
        }
        cx.notify();
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
        self.cursor = self.buffer.move_visual_up(self.cursor);
        cx.notify();
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }
        self.cursor = self.buffer.move_visual_down(self.cursor);
        cx.notify();
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        if let Some(line) = self.buffer.line(self.cursor.row) {
            if self.cursor.column == 0 {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                    self.cursor.column = self.buffer.line_len(self.cursor.row);
                }
                cx.notify();
                return;
            }

            let chars: Vec<char> = line.chars().collect();
            let mut char_pos = line[..self.cursor.column].chars().count();

            if char_pos == 0 {
                cx.notify();
                return;
            }

            char_pos -= 1;
            while char_pos > 0 && chars[char_pos].is_whitespace() {
                char_pos -= 1;
            }

            if char_pos > 0 {
                let is_alphanumeric = chars[char_pos].is_alphanumeric() || chars[char_pos] == '_';
                while char_pos > 0 {
                    let prev_char = chars[char_pos - 1];
                    let prev_is_alphanumeric = prev_char.is_alphanumeric() || prev_char == '_';
                    if is_alphanumeric != prev_is_alphanumeric || prev_char.is_whitespace() {
                        break;
                    }
                    char_pos -= 1;
                }
            }

            let byte_pos: usize = chars[..char_pos].iter().map(|c| c.len_utf8()).sum();
            self.cursor.column = byte_pos;
        }
        cx.notify();
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection_anchor.is_none() {
            self.selection_anchor = Some(self.cursor);
        }

        if let Some(line) = self.buffer.line(self.cursor.row) {
            if self.cursor.column >= line.len() {
                if self.cursor.row + 1 < self.buffer.line_count() {
                    self.cursor.row += 1;
                    self.cursor.column = 0;
                }
                cx.notify();
                return;
            }

            let after = &line[self.cursor.column..];
            let chars: Vec<char> = after.chars().collect();
            let mut char_pos = 0;

            if chars.is_empty() {
                cx.notify();
                return;
            }

            while char_pos < chars.len() && chars[char_pos].is_whitespace() {
                char_pos += 1;
            }

            if char_pos < chars.len() {
                let is_alphanumeric = chars[char_pos].is_alphanumeric() || chars[char_pos] == '_';
                while char_pos < chars.len() {
                    let curr_char = chars[char_pos];
                    let curr_is_alphanumeric = curr_char.is_alphanumeric() || curr_char == '_';
                    if is_alphanumeric != curr_is_alphanumeric || curr_char.is_whitespace() {
                        break;
                    }
                    char_pos += 1;
                }
            }

            let byte_offset: usize = chars[..char_pos].iter().map(|c| c.len_utf8()).sum();
            self.cursor.column += byte_offset;
        }
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.selection_anchor = Some(BufferPosition::zero());
        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_row);
        self.cursor = BufferPosition::new(last_row, last_col);
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            let start_offset = self.buffer.position_to_byte_offset(start);
            let end_offset = self.buffer.position_to_byte_offset(end);
            let content = self.buffer.to_string();
            if end_offset <= content.len() {
                let selected_text = content[start_offset..end_offset].to_string();
                cx.write_to_clipboard(selected_text.into());
            }
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            let start_offset = self.buffer.position_to_byte_offset(start);
            let end_offset = self.buffer.position_to_byte_offset(end);
            let content = self.buffer.to_string();
            if end_offset <= content.len() {
                self.push_undo_state();
                self.last_edit_time = None;
                let selected_text = content[start_offset..end_offset].to_string();
                cx.write_to_clipboard(selected_text.into());
                self.buffer.delete_range(start, end);
                self.cursor = start;
                self.clear_selection();
                cx.notify();
            }
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard_item) = cx.read_from_clipboard() {
            if let Some(text) = clipboard_item.text() {
                self.push_undo_state();
                self.last_edit_time = None;
                if let Some((start, end)) = self.selection_range() {
                    self.buffer.delete_range(start, end);
                    self.cursor = start;
                    self.clear_selection();
                }

                self.buffer.insert_str(self.cursor, &text);

                let newline_count = text.matches('\n').count();
                if newline_count > 0 {
                    let last_line = text.split('\n').last().unwrap_or("");
                    self.cursor = BufferPosition::new(self.cursor.row + newline_count, last_line.len());
                } else {
                    self.cursor.column += text.len();
                }

                cx.notify();
            }
        }
    }

    fn position_from_mouse(&mut self, mouse_position: Point<Pixels>, window: &mut Window, wrap_width: Pixels) -> BufferPosition {
        let line_height_px = px(self.font_size * 1.5);
        let padding_top = px(40.0);
        let padding_left = px(16.0);

        let relative_y = if mouse_position.y > padding_top {
            mouse_position.y - padding_top
        } else {
            px(0.0)
        };

        let relative_x = if mouse_position.x > padding_left {
            mouse_position.x - padding_left
        } else {
            px(0.0)
        };

        let visual_row = (relative_y / line_height_px) as usize;
        let text_system = window.text_system();
        let font_size_px = px(self.font_size);

        for buffer_row in 0..self.buffer.line_count() {
            self.buffer.get_or_shape_line(buffer_row, font_size_px, wrap_width, &text_system);
        }

        let mut visual_row_counter = 0;
        for buffer_row in 0..self.buffer.line_count() {
            if let Some(visual_lines) = self.buffer.get_visual_lines(buffer_row) {
                let visual_lines_vec: Vec<_> = visual_lines.iter().map(|vl| (vl.byte_range.clone(), vl.wrap_type)).collect();

                for (_idx, (byte_range, _wrap_type)) in visual_lines_vec.iter().enumerate() {
                    if visual_row_counter == visual_row {
                        if let Some(layout) = self.buffer.get_or_shape_line(buffer_row, font_size_px, wrap_width, &text_system) {
                            let full_line_x = layout.x_for_index(byte_range.start);
                            let relative_segment_x = relative_x + full_line_x;
                            let column_in_full_line = layout.closest_index_for_x(relative_segment_x);
                            let clamped_column = column_in_full_line.clamp(byte_range.start, byte_range.end);
                            return BufferPosition::new(buffer_row, clamped_column);
                        }
                    }
                    visual_row_counter += 1;
                }
            } else {
                if visual_row_counter == visual_row {
                    if let Some(layout) = self.buffer.get_or_shape_line(buffer_row, font_size_px, wrap_width, &text_system) {
                        let column = layout.closest_index_for_x(relative_x);
                        return BufferPosition::new(buffer_row, column);
                    }
                }
                visual_row_counter += 1;
            }
        }

        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line_len(last_row);
        BufferPosition::new(last_row, last_col)
    }

    fn find_word_boundaries(&self, pos: BufferPosition) -> Option<(BufferPosition, BufferPosition)> {
        let line = self.buffer.line(pos.row)?;
        if line.is_empty() || pos.column >= line.len() {
            return None;
        }

        let chars: Vec<char> = line.chars().collect();
        let char_indices: Vec<usize> = line.char_indices().map(|(i, _)| i).collect();

        let mut char_pos = 0;
        for (idx, &byte_idx) in char_indices.iter().enumerate() {
            if byte_idx >= pos.column {
                char_pos = idx;
                break;
            }
        }

        if char_pos >= chars.len() {
            return None;
        }

        let current_char = chars[char_pos];
        if !current_char.is_alphanumeric() && current_char != '_' {
            return None;
        }

        let mut start_char = char_pos;
        while start_char > 0 {
            let ch = chars[start_char - 1];
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            start_char -= 1;
        }

        let mut end_char = char_pos;
        while end_char < chars.len() {
            let ch = chars[end_char];
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            end_char += 1;
        }

        let start_byte = if start_char < char_indices.len() {
            char_indices[start_char]
        } else {
            line.len()
        };

        let end_byte = if end_char < char_indices.len() {
            char_indices[end_char]
        } else {
            line.len()
        };

        Some((
            BufferPosition::new(pos.row, start_byte),
            BufferPosition::new(pos.row, end_byte),
        ))
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);

        let window_size = window.viewport_size();
        let wrap_width = window_size.width - px(32.0);
        let position = self.position_from_mouse(event.position, window, wrap_width);

        let now = Instant::now();
        let is_double_click = if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_position) {
            now.duration_since(last_time) < DOUBLE_CLICK_DURATION && last_pos == position
        } else {
            false
        };

        if is_double_click {
            if let Some((start, end)) = self.find_word_boundaries(position) {
                self.selection_anchor = Some(start);
                self.cursor = end;
                self.is_dragging = false;
            } else {
                self.cursor = position;
                self.selection_anchor = Some(position);
                self.is_dragging = true;
            }
            self.last_click_time = None;
            self.last_click_position = None;
        } else {
            self.cursor = position;
            self.selection_anchor = Some(position);
            self.is_dragging = true;
            self.last_click_time = Some(now);
            self.last_click_position = Some(position);
        }

        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_dragging {
            let window_size = window.viewport_size();
            let wrap_width = window_size.width - px(32.0);
            let position = self.position_from_mouse(event.position, window, wrap_width);
            self.cursor = position;
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = false;
        if let Some(anchor) = self.selection_anchor {
            if anchor == self.cursor {
                self.clear_selection();
            }
        }
        cx.notify();
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(key_char) = &event.keystroke.key_char {
            if !event.keystroke.modifiers.platform
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt {
                self.push_undo_state();
                self.mark_edit_time();
                if let Some((start, end)) = self.selection_range() {
                    self.buffer.delete_range(start, end);
                    self.cursor = start;
                    self.clear_selection();
                }
                self.buffer.insert_str(self.cursor, key_char);
                self.cursor.column += key_char.len();
                cx.notify();
            }
        }
    }
}

impl Focusable for TextEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextEditor {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let font_size_px = px(self.font_size);
        let is_empty = self.buffer.line_count() == 1 && self.buffer.line_len(0) == 0;
        let window_size = _window.viewport_size();
        let wrap_width = window_size.width - px(32.0);

        div()
            .track_focus(&self.focus_handle)
            .on_action(_cx.listener(Self::increase_font_size))
            .on_action(_cx.listener(Self::decrease_font_size))
            .on_action(_cx.listener(Self::reset_font_size))
            .on_action(_cx.listener(Self::handle_newline))
            .on_action(_cx.listener(Self::handle_backspace))
            .on_action(_cx.listener(Self::handle_delete))
            .on_action(_cx.listener(Self::delete_to_beginning_of_line))
            .on_action(_cx.listener(Self::delete_to_end_of_line))
            .on_action(_cx.listener(Self::move_to_beginning_of_line))
            .on_action(_cx.listener(Self::move_to_end_of_line))
            .on_action(_cx.listener(Self::move_left))
            .on_action(_cx.listener(Self::move_right))
            .on_action(_cx.listener(Self::move_up))
            .on_action(_cx.listener(Self::move_down))
            .on_action(_cx.listener(Self::move_word_left))
            .on_action(_cx.listener(Self::move_word_right))
            .on_action(_cx.listener(Self::move_line_up))
            .on_action(_cx.listener(Self::move_line_down))
            .on_action(_cx.listener(Self::select_left))
            .on_action(_cx.listener(Self::select_right))
            .on_action(_cx.listener(Self::select_up))
            .on_action(_cx.listener(Self::select_down))
            .on_action(_cx.listener(Self::select_word_left))
            .on_action(_cx.listener(Self::select_word_right))
            .on_action(_cx.listener(Self::select_all))
            .on_action(_cx.listener(Self::copy))
            .on_action(_cx.listener(Self::cut))
            .on_action(_cx.listener(Self::paste))
            .on_action(_cx.listener(Self::undo))
            .on_action(_cx.listener(Self::redo))
            .on_action(_cx.listener(Self::delete_line))
            .on_action(_cx.listener(Self::handle_tab))
            .on_action(_cx.listener(Self::handle_outdent))
            .on_key_down(_cx.listener(Self::handle_key_down))
            .on_mouse_down(MouseButton::Left, _cx.listener(Self::handle_mouse_down))
            .on_mouse_move(_cx.listener(Self::handle_mouse_move))
            .on_mouse_up(MouseButton::Left, _cx.listener(Self::handle_mouse_up))
            .size_full()
            .bg(self.theme.background)
            .text_color(self.theme.text)
            .cursor(CursorStyle::IBeam)
            .pt_10()
            .px_4()
            .child(
                div()
                    .font_family("Monaco")
                    .text_size(font_size_px)
                    .line_height(relative(1.5))
                    .flex()
                    .flex_col()
                    .when(is_empty, |parent| {
                        parent.child(
                            div()
                                .relative()
                                .flex()
                                .items_center()
                                .child(
                                    div()
                                        .text_color(self.theme.text_muted)
                                        .child("Start typing...")
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(0.0))
                                        .top(px(0.0))
                                        .w(px(2.0))
                                        .h(font_size_px)
                                        .bg(self.theme.cursor)
                                )
                        )
                    })
                    .when(!is_empty, |parent| {
                        let selection_range = self.selection_range();
                        let mut container = parent;
                        let text_system = _window.text_system();

                        for row in 0..self.buffer.line_count() {
                            let line_text = self.buffer.line(row).unwrap_or("").to_string();

                            self.buffer.get_or_shape_line(row, font_size_px, wrap_width, &text_system);

                            if let Some(visual_lines) = self.buffer.get_visual_lines(row) {
                                let visual_lines_vec: Vec<_> = visual_lines.iter().map(|vl| (vl.byte_range.clone(), vl.wrap_type)).collect();

                                for (_visual_idx, (byte_range, wrap_type)) in visual_lines_vec.iter().enumerate() {
                                    let segment_text = &line_text[byte_range.clone()];
                                    let mut display_text = segment_text.to_string();

                                    if *wrap_type == WrapType::Hyphenated {
                                        display_text.push('-');
                                    }

                                    let is_cursor_on_this_segment = row == self.cursor.row
                                        && self.cursor.column >= byte_range.start
                                        && self.cursor.column <= byte_range.end;

                                    let mut line_div = div()
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .whitespace_nowrap()
                                        .child(StyledText::new(SharedString::from(display_text.clone())));

                                    if let Some((sel_start, sel_end)) = selection_range {
                                        if sel_start.row <= row && row <= sel_end.row {
                                            let seg_start = byte_range.start;
                                            let seg_end = byte_range.end;

                                            let line_start_col = if sel_start.row == row { sel_start.column } else { 0 };
                                            let line_end_col = if sel_end.row == row { sel_end.column } else { line_text.len() };

                                            let sel_start_in_seg = line_start_col.max(seg_start);
                                            let sel_end_in_seg = line_end_col.min(seg_end);

                                            if sel_start_in_seg < sel_end_in_seg {
                                                if let Some(shaped) = self.buffer.get_or_shape_line(row, font_size_px, wrap_width, &text_system) {
                                                    let seg_x_offset = shaped.x_for_index(seg_start);
                                                    let sel_x = shaped.x_for_index(sel_start_in_seg) - seg_x_offset;
                                                    let sel_end_x = shaped.x_for_index(sel_end_in_seg) - seg_x_offset;
                                                    let sel_width = sel_end_x - sel_x;

                                                    line_div = line_div.child(
                                                        div()
                                                            .absolute()
                                                            .left(sel_x)
                                                            .top(px(0.0))
                                                            .bottom(px(0.0))
                                                            .w(sel_width)
                                                            .bg(self.theme.selection)
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    if is_cursor_on_this_segment {
                                        if let Some(shaped) = self.buffer.get_or_shape_line(row, font_size_px, wrap_width, &text_system) {
                                            let seg_x_offset = shaped.x_for_index(byte_range.start);
                                            let cursor_x = shaped.x_for_index(self.cursor.column.min(line_text.len())) - seg_x_offset;

                                            line_div = line_div.child(
                                                div()
                                                    .absolute()
                                                    .left(cursor_x)
                                                    .top(px(0.0))
                                                    .bottom(px(0.0))
                                                    .w(px(2.0))
                                                    .bg(self.theme.cursor)
                                            );
                                        }
                                    }

                                    container = container.child(line_div);
                                }
                            }
                        }
                        container
                    })
            )
    }
}
