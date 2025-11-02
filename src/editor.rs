use crate::actions::*;
use crate::theme::AtomOneDark;
use gpui::prelude::*;
use gpui::*;

pub struct TextEditor {
    content: String,
    font_size: f32,
    cursor_position: usize,
    selection_anchor: Option<usize>,
    focus_handle: FocusHandle,
    theme: AtomOneDark,
    is_dragging: bool,
}

impl TextEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            content: String::new(),
            font_size: 24.0,
            cursor_position: 0,
            selection_anchor: None,
            focus_handle: cx.focus_handle(),
            theme: AtomOneDark::default(),
            is_dragging: false,
        }
    }

    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            if anchor < self.cursor_position {
                (anchor, self.cursor_position)
            } else {
                (self.cursor_position, anchor)
            }
        })
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn increase_font_size(&mut self, _: &IncreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size + 2.0).min(72.0);
        cx.notify();
    }

    fn decrease_font_size(&mut self, _: &DecreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = (self.font_size - 2.0).max(8.0);
        cx.notify();
    }

    fn reset_font_size(&mut self, _: &ResetFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = 24.0;
        cx.notify();
    }

    fn handle_newline(&mut self, _: &Newline, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            self.content.drain(start..end);
            self.cursor_position = start;
            self.clear_selection();
        }
        self.content.insert(self.cursor_position, '\n');
        self.cursor_position += 1;
        cx.notify();
    }

    fn handle_backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            self.content.drain(start..end);
            self.cursor_position = start;
            self.clear_selection();
        } else if self.cursor_position > 0 {
            self.content.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
        cx.notify();
    }

    fn handle_delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            self.content.drain(start..end);
            self.cursor_position = start;
            self.clear_selection();
        } else if self.cursor_position < self.content.len() {
            self.content.remove(self.cursor_position);
        }
        cx.notify();
    }

    fn delete_to_beginning_of_line(&mut self, _: &DeleteToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        let line_start = self.content[..self.cursor_position].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        self.content.drain(line_start..self.cursor_position);
        self.cursor_position = line_start;
        cx.notify();
    }

    fn delete_to_end_of_line(&mut self, _: &DeleteToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        let line_end = self.content[self.cursor_position..].find('\n').map(|pos| self.cursor_position + pos).unwrap_or(self.content.len());
        self.content.drain(self.cursor_position..line_end);
        cx.notify();
    }

    fn move_to_beginning_of_line(&mut self, _: &MoveToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor_position = self.content[..self.cursor_position].rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        cx.notify();
    }

    fn move_to_end_of_line(&mut self, _: &MoveToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        self.cursor_position = self.content[self.cursor_position..].find('\n').map(|pos| self.cursor_position + pos).unwrap_or(self.content.len());
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if self.cursor_position > 0 {
            let mut new_pos = self.cursor_position - 1;
            while new_pos > 0 && !self.content.is_char_boundary(new_pos) {
                new_pos -= 1;
            }
            self.cursor_position = new_pos;
            cx.notify();
        }
    }

    fn move_right(&mut self, _: &MoveRight, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if self.cursor_position < self.content.len() {
            let mut new_pos = self.cursor_position + 1;
            while new_pos < self.content.len() && !self.content.is_char_boundary(new_pos) {
                new_pos += 1;
            }
            self.cursor_position = new_pos;
            cx.notify();
        }
    }

    fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        let before_cursor = &self.content[..self.cursor_position];

        if let Some(current_line_start) = before_cursor.rfind('\n') {
            let current_line_start = current_line_start + 1;
            let column = self.cursor_position - current_line_start;

            let before_current_line = &self.content[..current_line_start.saturating_sub(1)];
            if let Some(prev_line_start) = before_current_line.rfind('\n') {
                let prev_line_start = prev_line_start + 1;
                let prev_line_end = current_line_start.saturating_sub(1);
                let prev_line_length = prev_line_end - prev_line_start;

                self.cursor_position = prev_line_start + column.min(prev_line_length);
            } else {
                let prev_line_length = current_line_start.saturating_sub(1);
                self.cursor_position = column.min(prev_line_length);
            }
            cx.notify();
        }
    }

    fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        let before_cursor = &self.content[..self.cursor_position];
        let after_cursor = &self.content[self.cursor_position..];

        let current_line_start = before_cursor.rfind('\n').map(|pos| pos + 1).unwrap_or(0);
        let column = self.cursor_position - current_line_start;

        if let Some(next_line_start_offset) = after_cursor.find('\n') {
            let next_line_start = self.cursor_position + next_line_start_offset + 1;

            if next_line_start < self.content.len() {
                let remaining = &self.content[next_line_start..];
                let next_line_end = remaining.find('\n')
                    .map(|pos| next_line_start + pos)
                    .unwrap_or(self.content.len());

                let next_line_length = next_line_end - next_line_start;
                self.cursor_position = next_line_start + column.min(next_line_length);
                cx.notify();
            }
        }
    }

    fn move_word_left(&mut self, _: &MoveWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if self.cursor_position == 0 {
            return;
        }

        let before_cursor = &self.content[..self.cursor_position];
        let chars: Vec<char> = before_cursor.chars().collect();
        let mut pos = chars.len();

        if pos == 0 {
            return;
        }

        pos -= 1;
        while pos > 0 && chars[pos].is_whitespace() {
            pos -= 1;
        }

        if pos > 0 {
            let is_alphanumeric = chars[pos].is_alphanumeric() || chars[pos] == '_';
            while pos > 0 {
                let prev_char = chars[pos - 1];
                let prev_is_alphanumeric = prev_char.is_alphanumeric() || prev_char == '_';
                if is_alphanumeric != prev_is_alphanumeric || prev_char.is_whitespace() {
                    break;
                }
                pos -= 1;
            }
        }

        let byte_pos: usize = chars[..pos].iter().map(|c| c.len_utf8()).sum();
        self.cursor_position = byte_pos;
        cx.notify();
    }

    fn move_word_right(&mut self, _: &MoveWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.clear_selection();
        if self.cursor_position >= self.content.len() {
            return;
        }

        let after_cursor = &self.content[self.cursor_position..];
        let chars: Vec<char> = after_cursor.chars().collect();
        let mut pos = 0;

        if chars.is_empty() {
            return;
        }

        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        if pos < chars.len() {
            let is_alphanumeric = chars[pos].is_alphanumeric() || chars[pos] == '_';
            while pos < chars.len() {
                let curr_char = chars[pos];
                let curr_is_alphanumeric = curr_char.is_alphanumeric() || curr_char == '_';
                if is_alphanumeric != curr_is_alphanumeric || curr_char.is_whitespace() {
                    break;
                }
                pos += 1;
            }
        }

        let byte_offset: usize = chars[..pos].iter().map(|c| c.len_utf8()).sum();
        self.cursor_position += byte_offset;
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.selection_anchor = Some(0);
        self.cursor_position = self.content.len();
        cx.notify();
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            let selected_text = self.content[start..end].to_string();
            cx.write_to_clipboard(selected_text.into());
        }
    }

    fn cut(&mut self, _: &Cut, _: &mut Window, cx: &mut Context<Self>) {
        if let Some((start, end)) = self.selection_range() {
            let selected_text = self.content[start..end].to_string();
            cx.write_to_clipboard(selected_text.into());
            self.content.drain(start..end);
            self.cursor_position = start;
            self.clear_selection();
            cx.notify();
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard_item) = cx.read_from_clipboard() {
            if let Some(text) = clipboard_item.text() {
                if let Some((start, end)) = self.selection_range() {
                    self.content.drain(start..end);
                    self.cursor_position = start;
                    self.clear_selection();
                }
                self.content.insert_str(self.cursor_position, &text);
                self.cursor_position += text.len();
                cx.notify();
            }
        }
    }

    fn position_from_mouse(&self, mouse_position: Point<Pixels>) -> usize {
        let char_width_px = px(self.font_size * 0.6);
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

        let line_index = (relative_y / line_height_px) as usize;
        let col_index = (relative_x / char_width_px) as usize;

        let lines: Vec<&str> = if self.content.is_empty() {
            vec![""]
        } else {
            self.content.split('\n').collect()
        };

        let clamped_line_index = line_index.min(lines.len().saturating_sub(1));

        let mut byte_offset = 0;
        for (idx, line) in lines.iter().enumerate() {
            if idx == clamped_line_index {
                let line_len = line.chars().count();
                let clamped_col = col_index.min(line_len);
                let char_offset: usize = line.chars().take(clamped_col).map(|c| c.len_utf8()).sum();
                return byte_offset + char_offset;
            }
            byte_offset += line.len() + 1;
        }

        self.content.len()
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = true;
        let position = self.position_from_mouse(event.position);
        self.cursor_position = position;
        self.selection_anchor = Some(position);
        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.is_dragging {
            let position = self.position_from_mouse(event.position);
            self.cursor_position = position;
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = false;
        if let Some(anchor) = self.selection_anchor {
            if anchor == self.cursor_position {
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
                if let Some((start, end)) = self.selection_range() {
                    self.content.drain(start..end);
                    self.cursor_position = start;
                    self.clear_selection();
                }
                self.content.insert_str(self.cursor_position, key_char);
                self.cursor_position += key_char.len();
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

        let lines: Vec<&str> = if self.content.is_empty() {
            vec![""]
        } else {
            self.content.split('\n').collect()
        };

        let mut byte_offset = 0;
        let mut cursor_line_idx = 0;
        let mut cursor_col = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            let line_len = line.len();
            if byte_offset + line_len >= self.cursor_position {
                cursor_line_idx = line_idx;
                cursor_col = self.cursor_position - byte_offset;
                break;
            }
            byte_offset += line_len + 1;
        }

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
            .on_action(_cx.listener(Self::select_all))
            .on_action(_cx.listener(Self::copy))
            .on_action(_cx.listener(Self::cut))
            .on_action(_cx.listener(Self::paste))
            .on_key_down(_cx.listener(Self::handle_key_down))
            .on_mouse_down(MouseButton::Left, _cx.listener(Self::handle_mouse_down))
            .on_mouse_move(_cx.listener(Self::handle_mouse_move))
            .on_mouse_up(MouseButton::Left, _cx.listener(Self::handle_mouse_up))
            .size_full()
            .bg(self.theme.background)
            .text_color(self.theme.text)
            .pt_10()
            .px_4()
            .child(
                div()
                    .font_family("Monaco")
                    .text_size(font_size_px)
                    .line_height(relative(1.5))
                    .flex()
                    .flex_col()
                    .when(self.content.is_empty(), |parent| {
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
                    .when(!self.content.is_empty(), |parent| {
                        let selection_range = self.selection_range();
                        let mut container = parent;
                        let mut line_byte_offset = 0;

                        for (line_idx, line) in lines.iter().enumerate() {
                            let is_cursor_line = line_idx == cursor_line_idx;
                            let line_start = line_byte_offset;
                            let line_end = line_byte_offset + line.len();

                            let mut line_div = div()
                                .relative()
                                .flex()
                                .items_center()
                                .whitespace_nowrap()
                                .child(line.to_string());

                            if let Some((sel_start, sel_end)) = selection_range {
                                if sel_start < line_end && sel_end > line_start {
                                    let sel_line_start = if sel_start > line_start { sel_start - line_start } else { 0 };
                                    let sel_line_end = if sel_end < line_end { sel_end - line_start } else { line.len() };

                                    let char_width = font_size_px * 0.6;
                                    let sel_x = char_width * sel_line_start as f32;
                                    let sel_width = char_width * (sel_line_end - sel_line_start) as f32;

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

                            if is_cursor_line {
                                let char_width = font_size_px * 0.6;
                                let cursor_x = char_width * cursor_col as f32;

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

                            container = container.child(line_div);
                            line_byte_offset += line.len() + 1;
                        }
                        container
                    })
            )
    }
}
