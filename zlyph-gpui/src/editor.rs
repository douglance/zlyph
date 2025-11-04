use crate::actions::*;
use crate::text_buffer::{BufferPosition, TextBuffer, WrapType};
use crate::theme::Theme;
use gpui::prelude::*;
use gpui::*;
use std::time::{Duration, Instant};
use zlyph_core::{EditorAction, EditorEngine};

pub struct TextEditor {
    engine: EditorEngine,
    buffer: TextBuffer,
    focus_handle: FocusHandle,
    theme: Theme,
    is_dragging: bool,
    last_click_time: Option<Instant>,
    last_click_position: Option<BufferPosition>,
    file_path: std::path::PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl TextEditor {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut engine = EditorEngine::new();
        let file_path = EditorEngine::default_file_path();

        // Load existing file if it exists
        let last_modified = if file_path.exists() {
            let _ = engine.load_from_file(&file_path);
            std::fs::metadata(&file_path).ok().and_then(|m| m.modified().ok())
        } else {
            None
        };

        let buffer = TextBuffer::from_string(engine.state().to_string());

        Self {
            engine,
            buffer,
            focus_handle: cx.focus_handle(),
            theme: Theme::default(),
            is_dragging: false,
            last_click_time: None,
            last_click_position: None,
            file_path,
            last_modified,
        }
    }

    fn sync_buffer_from_engine(&mut self) {
        let state = self.engine.state();
        self.buffer = TextBuffer::from_string(state.to_string());
    }

    fn save_to_file(&self) {
        let file_path = EditorEngine::default_file_path();
        let _ = self.engine.save_to_file(&file_path);
    }

    fn sync_and_save(&mut self) {
        self.sync_buffer_from_engine();
        self.save_to_file();
        // Update last modified time after save
        if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = metadata.modified() {
                self.last_modified = Some(modified);
            }
        }
    }

    fn check_and_reload(&mut self, cx: &mut Context<Self>) {
        if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = metadata.modified() {
                if self.last_modified.map_or(true, |last| modified > last) {
                    if self.engine.load_from_file(&self.file_path).is_ok() {
                        self.last_modified = Some(modified);
                        self.sync_buffer_from_engine();
                        cx.notify();
                    }
                }
            }
        }
    }

    fn get_cursor(&self) -> BufferPosition {
        let core_cursor = self.engine.state().cursor;
        BufferPosition::new(core_cursor.row, core_cursor.column)
    }

    fn set_cursor(&mut self, pos: BufferPosition) {
        self.engine.state_mut().cursor = zlyph_core::BufferPosition::new(pos.row, pos.column);
    }

    fn get_selection_anchor(&self) -> Option<BufferPosition> {
        self.engine.state().selection_anchor.map(|pos| BufferPosition::new(pos.row, pos.column))
    }

    fn set_selection_anchor(&mut self, pos: Option<BufferPosition>) {
        self.engine.state_mut().selection_anchor = pos.map(|p| zlyph_core::BufferPosition::new(p.row, p.column));
    }

    fn get_font_size(&self) -> f32 {
        self.engine.state().font_size
    }

    fn selection_range(&self) -> Option<(BufferPosition, BufferPosition)> {
        self.get_selection_anchor().map(|anchor| {
            let cursor = self.get_cursor();
            if anchor.row < cursor.row
                || (anchor.row == cursor.row && anchor.column < cursor.column)
            {
                (anchor, cursor)
            } else {
                (cursor, anchor)
            }
        })
    }

    // All action handlers delegate to engine
    fn undo(&mut self, _: &Undo, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Undo);
        self.sync_and_save();
        cx.notify();
    }

    fn redo(&mut self, _: &Redo, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Redo);
        self.sync_and_save();
        cx.notify();
    }

    fn increase_font_size(&mut self, _: &IncreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::IncreaseFontSize);
        self.buffer.invalidate_all_layouts();
        cx.notify();
    }

    fn decrease_font_size(&mut self, _: &DecreaseFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::DecreaseFontSize);
        self.buffer.invalidate_all_layouts();
        cx.notify();
    }

    fn reset_font_size(&mut self, _: &ResetFontSize, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::ResetFontSize);
        self.buffer.invalidate_all_layouts();
        cx.notify();
    }

    fn handle_newline(&mut self, _: &Newline, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Newline);
        self.sync_and_save();
        cx.notify();
    }

    fn handle_backspace(&mut self, _: &Backspace, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Backspace);
        self.sync_and_save();
        cx.notify();
    }

    fn handle_delete(&mut self, _: &Delete, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Delete);
        self.sync_and_save();
        cx.notify();
    }

    fn delete_to_beginning_of_line(&mut self, _: &DeleteToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::DeleteToBeginningOfLine);
        self.sync_and_save();
        cx.notify();
    }

    fn delete_to_end_of_line(&mut self, _: &DeleteToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::DeleteToEndOfLine);
        self.sync_and_save();
        cx.notify();
    }

    fn move_to_beginning_of_line(&mut self, _: &MoveToBeginningOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveToBeginningOfLine);
        cx.notify();
    }

    fn move_to_end_of_line(&mut self, _: &MoveToEndOfLine, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveToEndOfLine);
        cx.notify();
    }

    fn move_left(&mut self, _: &MoveLeft, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveLeft);
        cx.notify();
    }

    fn move_right(&mut self, _: &MoveRight, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveRight);
        cx.notify();
    }

    fn move_up(&mut self, _: &MoveUp, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveUp);
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveDown);
        cx.notify();
    }

    fn move_word_left(&mut self, _: &MoveWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveWordLeft);
        cx.notify();
    }

    fn move_word_right(&mut self, _: &MoveWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveWordRight);
        cx.notify();
    }

    fn move_line_up(&mut self, _: &MoveLineUp, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveLineUp);
        self.sync_and_save();
        cx.notify();
    }

    fn move_line_down(&mut self, _: &MoveLineDown, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::MoveLineDown);
        self.sync_and_save();
        cx.notify();
    }

    fn delete_line(&mut self, _: &DeleteLine, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::DeleteLine);
        self.sync_and_save();
        cx.notify();
    }

    fn handle_tab(&mut self, _: &Tab, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Tab);
        self.sync_and_save();
        cx.notify();
    }

    fn handle_outdent(&mut self, _: &Outdent, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::Outdent);
        self.sync_and_save();
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectLeft);
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectRight);
        cx.notify();
    }

    fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectUp);
        cx.notify();
    }

    fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectDown);
        cx.notify();
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectWordLeft);
        cx.notify();
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectWordRight);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _window: &mut Window, cx: &mut Context<Self>) {
        self.engine.handle_action(EditorAction::SelectAll);
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
                let selected_text = content[start_offset..end_offset].to_string();
                cx.write_to_clipboard(selected_text.into());
                self.engine.handle_action(EditorAction::Cut);
                self.sync_and_save();
                cx.notify();
            }
        }
    }

    fn paste(&mut self, _: &Paste, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(clipboard_item) = cx.read_from_clipboard() {
            if let Some(text) = clipboard_item.text() {
                self.engine.handle_action(EditorAction::Paste(text));
                self.sync_and_save();
                cx.notify();
            }
        }
    }

    fn position_from_mouse(&mut self, mouse_position: Point<Pixels>, window: &mut Window, wrap_width: Pixels) -> BufferPosition {
        let line_height_px = px(self.get_font_size() * 1.5);
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
        let font_size_px = px(self.get_font_size());

        for buffer_row in 0..self.buffer.line_count() {
            self.buffer.get_or_shape_line(buffer_row, font_size_px, wrap_width, &text_system);
        }

        let mut visual_row_counter = 0;
        for buffer_row in 0..self.buffer.line_count() {
            if let Some(visual_lines) = self.buffer.get_visual_lines(buffer_row) {
                let visual_lines_vec: Vec<_> = visual_lines
                    .iter()
                    .map(|vl| (vl.byte_range.clone(), vl.wrap_type))
                    .collect();

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

        Some((BufferPosition::new(pos.row, start_byte), BufferPosition::new(pos.row, end_byte)))
    }

    fn handle_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        const DOUBLE_CLICK_DURATION: Duration = Duration::from_millis(500);

        let window_size = window.viewport_size();
        let wrap_width = window_size.width - px(32.0);
        let position = self.position_from_mouse(event.position, window, wrap_width);

        let now = Instant::now();
        let is_double_click = if let (Some(last_time), Some(last_pos)) =
            (self.last_click_time, self.last_click_position)
        {
            now.duration_since(last_time) < DOUBLE_CLICK_DURATION && last_pos == position
        } else {
            false
        };

        if is_double_click {
            if let Some((start, end)) = self.find_word_boundaries(position) {
                self.set_selection_anchor(Some(start));
                self.set_cursor(end);
                self.is_dragging = false;
            } else {
                self.set_cursor(position);
                self.set_selection_anchor(Some(position));
                self.is_dragging = true;
            }
            self.last_click_time = None;
            self.last_click_position = None;
        } else {
            self.set_cursor(position);
            self.set_selection_anchor(Some(position));
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
            self.set_cursor(position);
            cx.notify();
        }
    }

    fn handle_mouse_up(&mut self, _event: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.is_dragging = false;
        if let Some(anchor) = self.get_selection_anchor() {
            if anchor == self.get_cursor() {
                self.set_selection_anchor(None);
            }
        }
        cx.notify();
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(key_char) = &event.keystroke.key_char {
            if !event.keystroke.modifiers.platform
                && !event.keystroke.modifiers.control
                && !event.keystroke.modifiers.alt
            {
                self.engine.handle_action(EditorAction::TypeString(key_char.clone()));
                self.sync_and_save();
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
        // Check for file changes on every render
        self.check_and_reload(_cx);

        let font_size_px = px(self.get_font_size());
        let cursor = self.get_cursor();
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
                                        .child("Start typing..."),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(0.0))
                                        .top(px(0.0))
                                        .w(px(2.0))
                                        .h(font_size_px)
                                        .bg(self.theme.cursor),
                                ),
                        )
                    })
                    .when(!is_empty, |parent| {
                        let selection_range = self.selection_range();
                        let mut container = parent;
                        let text_system = _window.text_system();

                        for row in 0..self.buffer.line_count() {
                            let line_text = self.buffer.line(row).unwrap_or("").to_string();

                            self.buffer.get_or_shape_line(
                                row,
                                font_size_px,
                                wrap_width,
                                &text_system,
                            );

                            if let Some(visual_lines) = self.buffer.get_visual_lines(row) {
                                let visual_lines_vec: Vec<_> = visual_lines
                                    .iter()
                                    .map(|vl| (vl.byte_range.clone(), vl.wrap_type))
                                    .collect();

                                for (_visual_idx, (byte_range, wrap_type)) in
                                    visual_lines_vec.iter().enumerate()
                                {
                                    let segment_text = &line_text[byte_range.clone()];
                                    let mut display_text = segment_text.to_string();

                                    if *wrap_type == WrapType::Hyphenated {
                                        display_text.push('-');
                                    }

                                    let is_cursor_on_this_segment = row == cursor.row
                                        && cursor.column >= byte_range.start
                                        && cursor.column <= byte_range.end;

                                    let mut line_div = div()
                                        .relative()
                                        .flex()
                                        .items_center()
                                        .whitespace_nowrap()
                                        .child(StyledText::new(SharedString::from(
                                            display_text.clone(),
                                        )));

                                    if let Some((sel_start, sel_end)) = selection_range {
                                        if sel_start.row <= row && row <= sel_end.row {
                                            let seg_start = byte_range.start;
                                            let seg_end = byte_range.end;

                                            let line_start_col = if sel_start.row == row {
                                                sel_start.column
                                            } else {
                                                0
                                            };
                                            let line_end_col = if sel_end.row == row {
                                                sel_end.column
                                            } else {
                                                line_text.len()
                                            };

                                            let sel_start_in_seg = line_start_col.max(seg_start);
                                            let sel_end_in_seg = line_end_col.min(seg_end);

                                            if sel_start_in_seg < sel_end_in_seg {
                                                if let Some(shaped) = self.buffer.get_or_shape_line(
                                                    row,
                                                    font_size_px,
                                                    wrap_width,
                                                    &text_system,
                                                ) {
                                                    let seg_x_offset =
                                                        shaped.x_for_index(seg_start);
                                                    let sel_x = shaped
                                                        .x_for_index(sel_start_in_seg)
                                                        - seg_x_offset;
                                                    let sel_end_x = shaped
                                                        .x_for_index(sel_end_in_seg)
                                                        - seg_x_offset;
                                                    let sel_width = sel_end_x - sel_x;

                                                    line_div = line_div.child(
                                                        div()
                                                            .absolute()
                                                            .left(sel_x)
                                                            .top(px(0.0))
                                                            .bottom(px(0.0))
                                                            .w(sel_width)
                                                            .bg(self.theme.selection),
                                                    );
                                                }
                                            }
                                        }
                                    }

                                    if is_cursor_on_this_segment {
                                        if let Some(shaped) = self.buffer.get_or_shape_line(
                                            row,
                                            font_size_px,
                                            wrap_width,
                                            &text_system,
                                        ) {
                                            let seg_x_offset = shaped.x_for_index(byte_range.start);
                                            let cursor_x = shaped.x_for_index(
                                                cursor.column.min(line_text.len()),
                                            ) - seg_x_offset;

                                            line_div = line_div.child(
                                                div()
                                                    .absolute()
                                                    .left(cursor_x)
                                                    .top(px(0.0))
                                                    .bottom(px(0.0))
                                                    .w(px(2.0))
                                                    .bg(self.theme.cursor),
                                            );
                                        }
                                    }

                                    container = container.child(line_div);
                                }
                            }
                        }
                        container
                    }),
            )
    }
}
