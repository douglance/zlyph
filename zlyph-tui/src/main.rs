use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Style},
    widgets::Paragraph,
    Terminal,
};
use zlyph_core::{EditorAction, EditorEngine};

struct TuiEditor {
    engine: EditorEngine,
}

impl TuiEditor {
    fn new() -> Self {
        let mut engine = EditorEngine::new();
        let file_path = EditorEngine::default_file_path();

        // Load existing file if it exists
        if file_path.exists() {
            let _ = engine.load_from_file(&file_path);
        }

        Self { engine }
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
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                if let Some(action) = self.translate_key_event(key) {
                    if matches!(action, EditorAction::Quit) {
                        // Save before quitting
                        let file_path = EditorEngine::default_file_path();
                        let _ = self.engine.save_to_file(&file_path);
                        break;
                    }
                    self.engine.handle_action(action);

                    // Auto-save after each action
                    let file_path = EditorEngine::default_file_path();
                    let _ = self.engine.save_to_file(&file_path);
                }
            }
        }
        Ok(())
    }

    fn translate_key_event(&self, event: KeyEvent) -> Option<EditorAction> {
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

            // Font size (will be ignored in TUI but kept for consistency)
            (KeyCode::Char('='), KeyModifiers::CONTROL) => Some(EditorAction::IncreaseFontSize),
            (KeyCode::Char('-'), KeyModifiers::CONTROL) => Some(EditorAction::DecreaseFontSize),

            // Ctrl+A for select all
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(EditorAction::SelectAll),

            // Tab/Outdent
            (KeyCode::Tab, KeyModifiers::SHIFT) => Some(EditorAction::Outdent),
            (KeyCode::Tab, KeyModifiers::NONE) => Some(EditorAction::Tab),

            // Alt arrow keys for moving lines (before selection and movement)
            (KeyCode::Up, KeyModifiers::ALT) => Some(EditorAction::MoveLineUp),
            (KeyCode::Down, KeyModifiers::ALT) => Some(EditorAction::MoveLineDown),

            // Selection with Shift (before regular movement)
            (KeyCode::Left, KeyModifiers::SHIFT) => Some(EditorAction::SelectLeft),
            (KeyCode::Right, KeyModifiers::SHIFT) => Some(EditorAction::SelectRight),
            (KeyCode::Up, KeyModifiers::SHIFT) => Some(EditorAction::SelectUp),
            (KeyCode::Down, KeyModifiers::SHIFT) => Some(EditorAction::SelectDown),

            // Cursor movement (after modifier versions)
            (KeyCode::Left, _) => Some(EditorAction::MoveLeft),
            (KeyCode::Right, _) => Some(EditorAction::MoveRight),
            (KeyCode::Up, _) => Some(EditorAction::MoveUp),
            (KeyCode::Down, _) => Some(EditorAction::MoveDown),
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

        // Build text with cursor indicator
        let mut display_lines = Vec::new();

        for (row_idx, line) in state.lines.iter().enumerate() {
            if row_idx == state.cursor.row {
                // Insert cursor on current line
                let (before, after) = if state.cursor.column <= line.len() {
                    line.split_at(state.cursor.column)
                } else {
                    (line.as_str(), "")
                };

                // Show selection if present
                let line_text = if let Some(anchor) = state.selection_anchor {
                    // Simple selection visualization
                    if anchor.row == row_idx && anchor.column != state.cursor.column {
                        let (start_col, end_col) = if anchor.column < state.cursor.column {
                            (anchor.column, state.cursor.column)
                        } else {
                            (state.cursor.column, anchor.column)
                        };

                        let before_sel = &line[..start_col];
                        let selected = &line[start_col..end_col.min(line.len())];
                        let after_sel = if end_col < line.len() { &line[end_col..] } else { "" };

                        format!("{}[{}]{}", before_sel, selected, after_sel)
                    } else {
                        format!("{}█{}", before, after)
                    }
                } else {
                    format!("{}█{}", before, after)
                };

                display_lines.push(line_text);
            } else {
                // Show selection on other lines if present
                if let Some(anchor) = state.selection_anchor {
                    let (sel_start_row, sel_end_row) = if anchor.row < state.cursor.row {
                        (anchor.row, state.cursor.row)
                    } else {
                        (state.cursor.row, anchor.row)
                    };

                    if row_idx >= sel_start_row && row_idx <= sel_end_row && sel_start_row != sel_end_row {
                        display_lines.push(format!("[{}]", line));
                    } else {
                        display_lines.push(line.clone());
                    }
                } else {
                    display_lines.push(line.clone());
                }
            }
        }

        let text = display_lines.join("\n");

        let paragraph = Paragraph::new(text)
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
