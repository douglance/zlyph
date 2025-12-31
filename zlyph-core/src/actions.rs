//! Platform-agnostic editor actions

#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    // Text manipulation
    TypeCharacter(char),
    TypeString(String),
    Backspace,
    Delete,
    Newline,
    Paste(String),

    // Cursor movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveToBeginningOfLine,
    MoveToEndOfLine,
    MoveWordLeft,
    MoveWordRight,

    // Selection
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectWordLeft,
    SelectWordRight,
    SelectAll,

    // Editing operations
    Undo,
    Redo,
    Cut,
    Copy,
    DeleteLine,
    DeleteToBeginningOfLine,
    DeleteToEndOfLine,
    DeleteWordLeft,
    DeleteWordRight,
    MoveLineUp,
    MoveLineDown,
    Tab,
    Outdent,

    // View operations
    IncreaseFontSize,
    DecreaseFontSize,
    ResetFontSize,

    // System operations
    Quit,

    // Mouse-driven cursor positioning
    SetCursorPosition { row: usize, column: usize },
    StartSelection { row: usize, column: usize },
    ExtendSelection { row: usize, column: usize },
}
