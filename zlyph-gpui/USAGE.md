# Zlyph Usage Guide

## Quick Start

```bash
# Install TUI (terminal) editor
cargo install --path zlyph-tui

# Run it
zlyph
```

The `zlyph` command opens your scratchpad at `~/.config/zlyph/default.txt`

## Shared State

Both the TUI and GPUI versions work on the **same file**: `~/.config/zlyph/default.txt`

- Changes auto-save after every keystroke
- Switch between editors seamlessly
- Your work is always persisted

## Running Both Editors

### Terminal (TUI)
```bash
zlyph              # Exit with Ctrl+W
```

### GUI (GPUI)  
```bash
cargo run -p zlyph-gpui
```

## Keyboard Shortcuts

All shortcuts work identically in both editors:

| Action | Shortcut |
|--------|----------|
| **Text** | |
| Type | Any key |
| New line | Enter |
| Delete backward | Backspace |
| Delete forward | Delete |
| **Cursor** | |
| Move | Arrow keys |
| Line start | Home |
| Line end | End |
| Word jump | Ctrl+Left/Right (GPUI only) |
| **Selection** | |
| Select | Shift+Arrows |
| Select all | Ctrl+A |
| **Editing** | |
| Undo | Ctrl+Z |
| Redo | Ctrl+Shift+Z |
| Delete line | Ctrl+Shift+K |
| Move line up | Alt+Up |
| Move line down | Alt+Down |
| Tab | Tab |
| Outdent | Shift+Tab |
| **View (GPUI only)** | |
| Bigger font | Ctrl+= |
| Smaller font | Ctrl+- |

## List Continuation

Smart list continuation works in both editors:

```
Type: - First item
Press Enter
→ Automatically creates: - 

Type: 1. First item
Press Enter  
→ Automatically creates: 2.
```

## Example Workflow

```bash
# Morning: Quick notes in terminal
zlyph
# Type meeting notes
# Ctrl+W to quit (auto-saved)

# Afternoon: Expand in GUI with nice font
cargo run -p zlyph-gpui
# Continue editing same file
# Close window (auto-saved)

# Evening: Review in terminal
zlyph
# All your work from both sessions is there
```

## File Location

Your scratchpad file: `~/.config/zlyph/default.txt`

You can directly edit this file or use either editor - they all stay in sync.
