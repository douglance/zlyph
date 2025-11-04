# Zlyph Features

## Shared Persistent State

Both editors operate on a single shared file: `~/.config/zlyph/default.txt`

- **Auto-save**: Every keystroke is saved immediately
- **Live reload**: Changes from other editors appear in real-time
- **No locks**: Simple last-write-wins semantics
- **Survives crashes**: All changes persisted instantly

## Dual Interface

### TUI (Terminal)
```bash
zlyph
```
- Minimal text-based interface
- Runs in any terminal
- Exit with `Ctrl+W`
- 100ms file polling for live updates

### GUI (GPUI)
```bash
cargo run -p zlyph-gpui
```
- Rich graphical interface
- Variable font size (`Ctrl+=` / `Ctrl+-`)
- Mouse support
- Real-time file watching (checks every frame)

## Smart Editing

### List Continuation
Type a list item and press Enter:
- `- item` → automatically creates `- `
- `1. item` → automatically creates `2. `
- `* item` → automatically creates `* `

### Text Selection
- `Shift+Arrows` - select text
- `Shift+Alt+Left/Right` - select word
- `Ctrl+A` - select all
- Works identically in both editors

### Navigation
- Arrow keys - move cursor
- `Alt+Left/Right` - jump by word
- `Cmd+Left/Right` - jump to line start/end (Mac)
- `Home/End` - line start/end

### Line Operations
- `Ctrl+Shift+K` - delete current line
- `Alt+Up` - move line up
- `Alt+Down` - move line down
- Tab/Shift+Tab - indent/outdent

### Undo/Redo
- Time-based chunking (500ms)
- Undo: `Ctrl+Z`
- Redo: `Ctrl+Shift+Z`
- Preserved per editor session

## Architecture

### zlyph-core
Platform-agnostic editing engine with 26 tests:
- EditorState: lines, cursor, selection, font size
- EditorEngine: business logic for all actions
- EditorAction: 28 action types
- File I/O: load_from_file, save_to_file

### zlyph-tui
Terminal UI using ratatui:
- Translates crossterm events to EditorActions
- Renders cursor as `█` character
- Selection shown with `[]` brackets
- Polls file every 100ms

### zlyph-gpui
GUI using GPUI framework:
- Complex TextBuffer with line wrapping
- Mouse support with drag selection
- Font size control
- Checks file on every render frame

## Real-Time Collaboration

Multiple instances can run simultaneously:

```bash
# Terminal 1
zlyph

# Terminal 2 (same time)
cargo run -p zlyph-gpui

# Terminal 3 (edit directly)
echo "Hello!" >> ~/.config/zlyph/default.txt
```

All instances stay synchronized automatically.

## Implementation Details

**File Watching**:
- TUI: `poll(100ms)` with modification time check
- GPUI: Check on every render (~60 FPS)
- Both track `last_modified` timestamp
- Reload when `file_modified > last_modified`
- Update timestamp after saving

**Conflict Resolution**:
- No pessimistic locking
- Last write wins
- OS provides atomic write guarantees
- Acceptable for single-user scratchpad use case

**Performance**:
- File checks are cheap (stat syscall)
- Reload only when timestamp changes
- No unnecessary re-renders
- TUI: 100ms latency max
- GPUI: ~16ms latency (60 FPS)
