use zlyph_core::{BufferPosition, EditorAction, EditorEngine};

#[test]
fn test_type_character() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('h'));
    engine.handle_action(EditorAction::TypeCharacter('i'));

    assert_eq!(engine.state().to_string(), "hi");
    assert_eq!(engine.state().cursor, BufferPosition::new(0, 2));
}

#[test]
fn test_type_string() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello world".to_string()));

    assert_eq!(engine.state().to_string(), "hello world");
    assert_eq!(engine.state().cursor.column, 11);
}

#[test]
fn test_backspace_empty() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::Backspace);

    assert_eq!(engine.state().to_string(), "");
    assert_eq!(engine.state().cursor, BufferPosition::zero());
}

#[test]
fn test_backspace_deletes_character() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('a'));
    engine.handle_action(EditorAction::TypeCharacter('b'));
    engine.handle_action(EditorAction::Backspace);

    assert_eq!(engine.state().to_string(), "a");
    assert_eq!(engine.state().cursor.column, 1);
}

#[test]
fn test_delete_character() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("abc".to_string()));
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::Delete);

    assert_eq!(engine.state().to_string(), "ab");
}

#[test]
fn test_move_cursor_left() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('a'));
    engine.handle_action(EditorAction::TypeCharacter('b'));
    engine.handle_action(EditorAction::MoveLeft);

    assert_eq!(engine.state().cursor.column, 1);
}

#[test]
fn test_move_cursor_left_at_start() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::MoveLeft);

    assert_eq!(engine.state().cursor, BufferPosition::zero());
}

#[test]
fn test_move_cursor_right() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('a'));
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::MoveRight);

    assert_eq!(engine.state().cursor.column, 1);
}

#[test]
fn test_newline() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("world".to_string()));

    assert_eq!(engine.state().to_string(), "hello\nworld");
    assert_eq!(engine.state().cursor.row, 1);
    assert_eq!(engine.state().cursor.column, 5);
}

#[test]
fn test_move_up_down() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("line1".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("line2".to_string()));

    assert_eq!(engine.state().cursor.row, 1);

    engine.handle_action(EditorAction::MoveUp);
    assert_eq!(engine.state().cursor.row, 0);

    engine.handle_action(EditorAction::MoveDown);
    assert_eq!(engine.state().cursor.row, 1);
}

#[test]
fn test_move_to_line_start_end() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello world".to_string()));

    engine.handle_action(EditorAction::MoveToBeginningOfLine);
    assert_eq!(engine.state().cursor.column, 0);

    engine.handle_action(EditorAction::MoveToEndOfLine);
    assert_eq!(engine.state().cursor.column, 11);
}

#[test]
fn test_undo_redo() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('a'));

    assert_eq!(engine.state().to_string(), "a");

    engine.handle_action(EditorAction::Undo);
    assert_eq!(engine.state().to_string(), "");

    engine.handle_action(EditorAction::Redo);
    assert_eq!(engine.state().to_string(), "a");
}

#[test]
fn test_undo_multiple_edits() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('a'));
    std::thread::sleep(std::time::Duration::from_millis(600));
    engine.handle_action(EditorAction::TypeCharacter('b'));
    std::thread::sleep(std::time::Duration::from_millis(600));
    engine.handle_action(EditorAction::TypeCharacter('c'));

    assert_eq!(engine.state().to_string(), "abc");

    engine.handle_action(EditorAction::Undo);
    assert_eq!(engine.state().to_string(), "ab");

    engine.handle_action(EditorAction::Undo);
    assert_eq!(engine.state().to_string(), "a");

    engine.handle_action(EditorAction::Undo);
    assert_eq!(engine.state().to_string(), "");
}

#[test]
fn test_unicode_handling() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeCharacter('🚀'));

    assert_eq!(engine.state().to_string(), "🚀");
    assert_eq!(engine.state().cursor.column, 4); // UTF-8 bytes
}

#[test]
fn test_increase_decrease_font_size() {
    let mut engine = EditorEngine::new();
    let initial_size = engine.state().font_size;

    engine.handle_action(EditorAction::IncreaseFontSize);
    assert_eq!(engine.state().font_size, initial_size + 2.0);

    engine.handle_action(EditorAction::DecreaseFontSize);
    assert_eq!(engine.state().font_size, initial_size);
}

#[test]
fn test_decrease_font_size_minimum() {
    let mut engine = EditorEngine::new();

    for _ in 0..30 {
        engine.handle_action(EditorAction::DecreaseFontSize);
    }

    assert!(engine.state().font_size >= 8.0);
}

#[test]
fn test_delete_line() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("line1".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("line2".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("line3".to_string()));

    engine.handle_action(EditorAction::MoveUp);
    engine.handle_action(EditorAction::DeleteLine);

    assert_eq!(engine.state().to_string(), "line1\nline3");
}

#[test]
fn test_move_line_up() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("line1".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("line2".to_string()));

    engine.handle_action(EditorAction::MoveLineUp);

    assert_eq!(engine.state().to_string(), "line2\nline1");
    assert_eq!(engine.state().cursor.row, 0);
}

#[test]
fn test_move_line_down() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("line1".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("line2".to_string()));
    engine.handle_action(EditorAction::MoveUp);

    engine.handle_action(EditorAction::MoveLineDown);

    assert_eq!(engine.state().to_string(), "line2\nline1");
    assert_eq!(engine.state().cursor.row, 1);
}

#[test]
fn test_tab_indentation() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello".to_string()));
    engine.handle_action(EditorAction::Tab);
    engine.handle_action(EditorAction::TypeString("world".to_string()));

    assert_eq!(engine.state().to_string(), "hello    world");
}

#[test]
fn test_select_all() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello\nworld".to_string()));
    engine.handle_action(EditorAction::SelectAll);

    assert!(engine.state().selection_anchor.is_some());
    assert_eq!(
        engine.state().selection_anchor.unwrap(),
        BufferPosition::zero()
    );
    assert_eq!(engine.state().cursor.row, 1);
    assert_eq!(engine.state().cursor.column, 5);
}

#[test]
fn test_select_left_right() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello".to_string()));
    engine.handle_action(EditorAction::SelectLeft);
    engine.handle_action(EditorAction::SelectLeft);

    assert!(engine.state().selection_anchor.is_some());
    assert_eq!(engine.state().cursor.column, 3);
}

#[test]
fn test_backspace_joins_lines() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("world".to_string()));
    engine.handle_action(EditorAction::MoveToBeginningOfLine);
    engine.handle_action(EditorAction::Backspace);

    assert_eq!(engine.state().to_string(), "helloworld");
    assert_eq!(engine.state().cursor.row, 0);
    assert_eq!(engine.state().cursor.column, 5);
}

#[test]
fn test_delete_joins_lines() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello".to_string()));
    engine.handle_action(EditorAction::Newline);
    engine.handle_action(EditorAction::TypeString("world".to_string()));
    engine.handle_action(EditorAction::MoveUp);
    engine.handle_action(EditorAction::MoveToEndOfLine);
    engine.handle_action(EditorAction::Delete);

    assert_eq!(engine.state().to_string(), "helloworld");
}

#[test]
fn test_delete_to_beginning_of_line() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello world".to_string()));
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::MoveLeft);
    engine.handle_action(EditorAction::DeleteToBeginningOfLine);

    assert_eq!(engine.state().to_string(), "world");
}

#[test]
fn test_delete_to_end_of_line() {
    let mut engine = EditorEngine::new();
    engine.handle_action(EditorAction::TypeString("hello world".to_string()));
    engine.handle_action(EditorAction::MoveToBeginningOfLine);
    engine.handle_action(EditorAction::MoveRight);
    engine.handle_action(EditorAction::MoveRight);
    engine.handle_action(EditorAction::MoveRight);
    engine.handle_action(EditorAction::MoveRight);
    engine.handle_action(EditorAction::MoveRight);
    engine.handle_action(EditorAction::DeleteToEndOfLine);

    assert_eq!(engine.state().to_string(), "hello");
}
