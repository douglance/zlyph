use anyhow::Result;
use crossterm::{
    event::{self, poll, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Terminal,
};
use std::time::Duration;
use zlyph_core::{EditorAction, EditorEngine};

struct TuiEditor {
    engine: EditorEngine,
    file_path: std::path::PathBuf,
    last_modified: Option<std::time::SystemTime>,
}

impl TuiEditor {
    fn new() -> Self {
        let mut engine = EditorEngine::new();
        let file_path = EditorEngine::default_file_path();

        // Load existing file if it exists
        let last_modified = if file_path.exists() {
            let _ = engine.load_from_file(&file_path);
            std::fs::metadata(&file_path).ok().and_then(|m| m.modified().ok())
        } else {
            None
        };

        Self {
            engine,
            file_path,
            last_modified,
        }
    }

    fn check_and_reload(&mut self) -> bool {
        if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = metadata.modified() {
                if self.last_modified.map_or(true, |last| modified > last) {
                    if self.engine.load_from_file(&self.file_path).is_ok() {
                        self.last_modified = Some(modified);
                        return true;
                    }
                }
            }
        }
        false
    }

    fn render_cursor_line<'a>(&self, line: &'a str, cursor_col: usize, spans: &mut Vec<Span<'a>>, cursor_style: Style) {
        if cursor_col == 0 {
            // Cursor at start
            if line.is_empty() {
                spans.push(Span::styled(" ", cursor_style));
            } else {
                let cursor_char = line.chars().next().unwrap();
                spans.push(Span::styled(cursor_char.to_string(), cursor_style));
                spans.push(Span::raw(&line[cursor_char.len_utf8()..]));
            }
        } else if cursor_col >= line.len() {
            // Cursor at end
            spans.push(Span::raw(line));
            spans.push(Span::styled(" ", cursor_style));
        } else {
            // Cursor in middle
            let (before, rest) = line.split_at(cursor_col);
            let cursor_char = rest.chars().next().unwrap();
            let after = &rest[cursor_char.len_utf8()..];

            spans.push(Span::raw(before));
            spans.push(Span::styled(cursor_char.to_string(), cursor_style));
            spans.push(Span::raw(after));
        }
    }

    fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        result
    }

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        loop {
            // Check for file changes before rendering
            if self.check_and_reload() {
                // File was reloaded
            }

            terminal.draw(|frame| self.render(frame))?;

            // Poll for events with timeout to check file changes periodically
            if poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if let Some(action) = self.translate_key_event(key) {
                        if matches!(action, EditorAction::Quit) {
                            // Save before quitting
                            let _ = self.engine.save_to_file(&self.file_path);
                            break;
                        }
                        self.engine.handle_action(action);

                        // Auto-save after each action
                        if self.engine.save_to_file(&self.file_path).is_ok() {
                            // Update last modified time after we save
                            if let Ok(metadata) = std::fs::metadata(&self.file_path) {
                                if let Ok(modified) = metadata.modified() {
                                    self.last_modified = Some(modified);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn translate_key_event(&self, event: KeyEvent) -> Option<EditorAction> {
        // Debug: See what keys terminal sends
        eprintln!("Key: {:?}, Mods: {:?}", event.code, event.modifiers);

        match (event.code, event.modifiers) {
            // Ctrl+W to quit
            (KeyCode::Char('w'), KeyModifiers::CONTROL) => Some(EditorAction::Quit),

            // Undo/Redo
            (KeyCode::Char('z'), mods) if mods.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Some(EditorAction::Redo)
            }
            (KeyCode::Char('z'), KeyModifiers::CONTROL) => Some(EditorAction::Undo),

            // Line operations
            (KeyCode::Char('k'), mods) if mods.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                Some(EditorAction::DeleteLine)
            }

            // Delete operations
            (KeyCode::Backspace, KeyModifiers::SUPER) => Some(EditorAction::DeleteLine),
            (KeyCode::Backspace, KeyModifiers::CONTROL) => Some(EditorAction::DeleteToBeginningOfLine),
            (KeyCode::Backspace, KeyModifiers::ALT) => Some(EditorAction::DeleteWordLeft),
            (KeyCode::Delete, KeyModifiers::SUPER) => Some(EditorAction::DeleteToEndOfLine),
            (KeyCode::Delete, KeyModifiers::CONTROL) => Some(EditorAction::DeleteToEndOfLine),
            (KeyCode::Delete, KeyModifiers::ALT) => Some(EditorAction::DeleteWordRight),

            // Font size (will be ignored in TUI but kept for consistency)
            (KeyCode::Char('='), KeyModifiers::CONTROL) => Some(EditorAction::IncreaseFontSize),
            (KeyCode::Char('-'), KeyModifiers::CONTROL) => Some(EditorAction::DecreaseFontSize),

            // Ctrl+A for select all
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(EditorAction::SelectAll),

            // Tab/Outdent
            (KeyCode::Tab, KeyModifiers::SHIFT) => Some(EditorAction::Outdent),
            (KeyCode::Tab, KeyModifiers::NONE) => Some(EditorAction::Tab),

            // Cmd+Left/Right for line start/end (Mac)
            (KeyCode::Left, KeyModifiers::SUPER) => Some(EditorAction::MoveToBeginningOfLine),
            (KeyCode::Right, KeyModifiers::SUPER) => Some(EditorAction::MoveToEndOfLine),

            // Alt+Left/Right for word jumping (check before shift combinations)
            (KeyCode::Left, mods) if mods == KeyModifiers::ALT => Some(EditorAction::MoveWordLeft),
            (KeyCode::Right, mods) if mods == KeyModifiers::ALT => Some(EditorAction::MoveWordRight),

            // Shift+Alt for word selection
            (KeyCode::Left, mods) if mods.contains(KeyModifiers::SHIFT) && mods.contains(KeyModifiers::ALT) => {
                Some(EditorAction::SelectWordLeft)
            }
            (KeyCode::Right, mods) if mods.contains(KeyModifiers::SHIFT) && mods.contains(KeyModifiers::ALT) => {
                Some(EditorAction::SelectWordRight)
            }

            // Alt+Up/Down for moving lines
            (KeyCode::Up, mods) if mods == KeyModifiers::ALT => Some(EditorAction::MoveLineUp),
            (KeyCode::Down, mods) if mods == KeyModifiers::ALT => Some(EditorAction::MoveLineDown),

            // Selection with Shift (before regular movement)
            (KeyCode::Left, KeyModifiers::SHIFT) => Some(EditorAction::SelectLeft),
            (KeyCode::Right, KeyModifiers::SHIFT) => Some(EditorAction::SelectRight),
            (KeyCode::Up, KeyModifiers::SHIFT) => Some(EditorAction::SelectUp),
            (KeyCode::Down, KeyModifiers::SHIFT) => Some(EditorAction::SelectDown),

            // Cursor movement (after modifier versions)
            (KeyCode::Left, KeyModifiers::NONE) => Some(EditorAction::MoveLeft),
            (KeyCode::Right, KeyModifiers::NONE) => Some(EditorAction::MoveRight),
            (KeyCode::Up, KeyModifiers::NONE) => Some(EditorAction::MoveUp),
            (KeyCode::Down, KeyModifiers::NONE) => Some(EditorAction::MoveDown),
            (KeyCode::Home, _) => Some(EditorAction::MoveToBeginningOfLine),
            (KeyCode::End, _) => Some(EditorAction::MoveToEndOfLine),

            // Text editing
            (KeyCode::Backspace, _) => Some(EditorAction::Backspace),
            (KeyCode::Delete, _) => Some(EditorAction::Delete),
            (KeyCode::Enter, _) => Some(EditorAction::Newline),

            // Regular character input
            (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                Some(EditorAction::TypeCharacter(c))
            }

            _ => None,
        }
    }

    fn render(&self, frame: &mut ratatui::Frame) {
        let state = self.engine.state();

        // Selection highlighting style
        let selection_style = Style::default().bg(Color::DarkGray);
        let cursor_style = Style::default().add_modifier(Modifier::REVERSED);

        // Build styled lines with cursor and selection highlighting
        let mut display_lines = Vec::new();

        for (row_idx, line) in state.lines.iter().enumerate() {
            let mut spans = Vec::new();

            if let Some(anchor) = state.selection_anchor {
                // Calculate selection range
                let (sel_start_row, sel_start_col, sel_end_row, sel_end_col) =
                    if anchor.row < state.cursor.row || (anchor.row == state.cursor.row && anchor.column < state.cursor.column) {
                        (anchor.row, anchor.column, state.cursor.row, state.cursor.column)
                    } else {
                        (state.cursor.row, state.cursor.column, anchor.row, anchor.column)
                    };

                if row_idx == state.cursor.row && row_idx >= sel_start_row && row_idx <= sel_end_row {
                    // Line with cursor and possibly selection
                    let (sel_from, sel_to) = if row_idx == sel_start_row && row_idx == sel_end_row {
                        (sel_start_col, sel_end_col)
                    } else if row_idx == sel_start_row {
                        (sel_start_col, line.len())
                    } else if row_idx == sel_end_row {
                        (0, sel_end_col)
                    } else {
                        (0, line.len())
                    };

                    // Before selection
                    if sel_from > 0 {
                        spans.push(Span::raw(&line[..sel_from]));
                    }

                    // Selected text
                    if sel_to > sel_from {
                        let sel_text = &line[sel_from..sel_to.min(line.len())];
                        spans.push(Span::styled(sel_text, selection_style));
                    }

                    // After selection
                    if sel_to < line.len() {
                        spans.push(Span::raw(&line[sel_to..]));
                    }

                    // Cursor
                    let cursor_col = state.cursor.column;
                    if cursor_col >= line.len() {
                        spans.push(Span::styled(" ", cursor_style));
                    }
                } else if row_idx >= sel_start_row && row_idx <= sel_end_row {
                    // Line within selection range but not cursor line
                    let (sel_from, sel_to) = if row_idx == sel_start_row {
                        (sel_start_col, line.len())
                    } else if row_idx == sel_end_row {
                        (0, sel_end_col)
                    } else {
                        (0, line.len())
                    };

                    if sel_from > 0 {
                        spans.push(Span::raw(&line[..sel_from]));
                    }
                    if sel_to > sel_from {
                        spans.push(Span::styled(&line[sel_from..sel_to.min(line.len())], selection_style));
                    }
                    if sel_to < line.len() {
                        spans.push(Span::raw(&line[sel_to..]));
                    }
                } else if row_idx == state.cursor.row {
                    // Cursor line without selection
                    self.render_cursor_line(line, state.cursor.column, &mut spans, cursor_style);
                } else {
                    // Regular line
                    spans.push(Span::raw(line.as_str()));
                }
            } else if row_idx == state.cursor.row {
                // No selection, just cursor
                self.render_cursor_line(line, state.cursor.column, &mut spans, cursor_style);
            } else {
                // Regular line
                spans.push(Span::raw(line.as_str()));
            }

            display_lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(display_lines)
            .style(Style::default().fg(Color::White));

        // Create a rect with padding on all sides
        let area = frame.size();
        let padded_area = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };

        frame.render_widget(paragraph, padded_area);
    }
}

fn main() -> Result<()> {
    let mut editor = TuiEditor::new();
    editor.run()
}
