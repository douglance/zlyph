# Zlyph Features

## Overview

Zlyph is a minimal text editor with both Terminal UI (TUI) and GUI (GPUI) interfaces. Both interfaces share the same editing engine and operate on a single persistent file.

## Shared State

**File Location**: `~/.config/zlyph/default.txt`

Both editors automatically:
- Load this file on startup
- Save after every edit action
- Reload when the file changes externally (live sync)

**TUI Reload**: Polls every 100ms checking modification time
**GPUI Reload**: Checks on every render frame

## Keyboard Shortcuts

### Navigation

| Shortcut | Action |
|----------|--------|
| `↑ ↓ ← →` | Move cursor |
| `Home` / `Cmd+Left` | Move to beginning of line |
| `End` / `Cmd+Right` | Move to end of line |
| `Alt+Left` | Move word left |
| `Alt+Right` | Move word right |
| `Alt+Up` | Move line up |
| `Alt+Down` | Move line down |

### Selection

| Shortcut | Action |
|----------|--------|
| `Shift+Arrow` | Select characters |
| `Shift+Alt+Arrow` | Select words |
| `Ctrl+A` | Select all |

### Editing

| Shortcut | Action |
|----------|--------|
| `Backspace` | Delete character left |
| `Delete` | Delete character right |
| `Ctrl+Backspace` | Delete to beginning of line |
| `Ctrl+Delete` | Delete to end of line |
| `Alt+Backspace` | Delete word left |
| `Alt+Delete` | Delete word right |
| `Cmd+Backspace` | Delete entire line |
| `Cmd+Delete` | Delete to end of line |
| `Ctrl+Shift+K` | Delete line |
| `Tab` | Insert tab (4 spaces) |
| `Shift+Tab` | Outdent |

### Undo/Redo

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |

### System

| Shortcut | Action |
|----------|--------|
| `Ctrl+W` | Quit (TUI) |

## Visual Features

### Cursor
- **Style**: Reverse video highlighting (white on black)
- **No extra space**: Cursor overlays existing character

### Selection
- **Style**: Dark gray background
- **Visibility**: Works across single and multi-line selections
- **Interaction**: Cursor can be within selection (both effects visible)

## Terminal Configuration

### Alt+Arrow Keys Not Working?

**Terminal.app**:
1. Preferences → Profiles → Keyboard
2. Check "Use Option as Meta key"

**iTerm2**:
1. Preferences → Profiles → Keys
2. Set "Left Option key" to "Esc+"

### Verify Terminal Sends Correct Keys

Enable debug mode (see KEYBOARD_DEBUG.md):
- Working: `Key: Left, Mods: ALT`
- Not working: `Key: Char('b'), Mods: ALT`

## Installation

```bash
# Install TUI globally as 'zlyph'
cargo install --path zlyph-tui

# Run from anywhere
zlyph
```

## Architecture

```
zlyph/
├── zlyph-core/      # Shared editing engine
│   ├── engine.rs    # EditorEngine with all logic
│   └── actions.rs   # Platform-agnostic actions
├── zlyph-tui/       # Terminal interface
│   └── main.rs      # Ratatui implementation
└── zlyph-gpui/      # GUI interface
    └── editor.rs    # GPUI implementation
```

All editing logic lives in `zlyph-core` to ensure identical behavior across interfaces.
