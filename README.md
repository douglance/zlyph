# Zlyph

A minimal text editor with both Terminal UI (TUI) and GUI interfaces. Both interfaces share the same editing engine and operate on a single persistent file for seamless synchronization.

## Quick Start

```bash
# Install globally
cargo install --path zlyph-tui

# Run from anywhere
zlyph
```

Both editors automatically sync via `~/.config/zlyph/default.txt`.

## Features

- **Dual Interface**: Terminal UI (Ratatui) and GUI (GPUI)
- **Shared State**: Both editors operate on single file with live reloading
- **Complete Editing**: Full navigation, selection, deletion with Mac-style shortcuts
- **Undo/Redo**: Intelligent change tracking
- **Line Operations**: Move, delete, duplicate lines
- **Word Operations**: Navigate and delete by word boundaries
- **Auto-Save**: Every edit instantly persisted
- **Visual Selection**: Clear highlighting for selected text

## Architecture

```
zlyph-core    # Shared editing engine (26 tests passing)
zlyph-tui     # Terminal interface (Ratatui)
zlyph-gpui    # GUI interface (GPUI)
```

All editing logic lives in `zlyph-core` to ensure identical behavior.

## Keyboard Shortcuts

### Navigation
- `↑ ↓ ← →` - Move cursor
- `Cmd+Left` / `Home` - Beginning of line
- `Cmd+Right` / `End` - End of line
- `Alt+Left` - Move word left
- `Alt+Right` - Move word right
- `Alt+Up` - Move line up
- `Alt+Down` - Move line down

### Selection
- `Shift+Arrow` - Select characters
- `Shift+Alt+Arrow` - Select words
- `Ctrl+A` - Select all

### Editing
- `Backspace` - Delete left
- `Delete` - Delete right
- `Alt+Backspace` - Delete word left
- `Alt+Delete` - Delete word right
- `Ctrl+Backspace` - Delete to line start
- `Ctrl+Delete` - Delete to line end
- `Cmd+Backspace` - Delete entire line
- `Ctrl+Shift+K` - Delete line
- `Tab` / `Shift+Tab` - Indent / Outdent

### Undo/Redo
- `Ctrl+Z` - Undo
- `Ctrl+Shift+Z` - Redo

### System
- `Ctrl+W` - Quit

## Terminal Configuration

**If Alt+arrow keys don't work:**

**Terminal.app**: Preferences → Profiles → Keyboard → Check "Use Option as Meta key"

**iTerm2**: Preferences → Profiles → Keys → Set "Left Option key" to "Esc+"

See `KEYBOARD_DEBUG.md` for troubleshooting.

## Building from Source

```bash
git clone https://github.com/douglance/zlyph.git
cd zlyph
cargo build

# Run specific interface
cargo run -p zlyph-tui
cargo run -p zlyph-gpui

# Run tests
cargo test -p zlyph-core
```

## Documentation

- `FEATURES.md` - Complete feature list
- `KEYBOARD_DEBUG.md` - Keyboard troubleshooting guide
- `TEST_SELECTIONS.md` - How to test selections
- `STATUS.md` - Project status and known issues
- `CLAUDE.md` - Development guidelines

## License

MIT
