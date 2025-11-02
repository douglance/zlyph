# Zlyph

A minimal text editor built with GPUI (the GUI framework from Zed).

## Features

- Text editing with cursor navigation
- Undo/redo with intelligent chunking
- Line manipulation (delete, duplicate)
- Text selection
- Auto-indentation
- List auto-continuation
- Font size adjustment

## Installation

```bash
cargo install zlyph
```

## Building from Source

```bash
git clone https://github.com/douglance/zlyph.git
cd zlyph
cargo build --release
```

## Usage

```bash
zlyph
```

## Keybindings

| Action | Key |
|--------|-----|
| Increase font size | Cmd/Ctrl + = |
| Decrease font size | Cmd/Ctrl + - |
| Delete line | Cmd/Ctrl + Shift + K |
| Duplicate line | Cmd/Ctrl + Shift + D |
| Undo | Cmd/Ctrl + Z |
| Redo | Cmd/Ctrl + Shift + Z |
| Move line up | Alt + ↑ |
| Move line down | Alt + ↓ |
| Select all | Cmd/Ctrl + A |
| Cut | Cmd/Ctrl + X |
| Copy | Cmd/Ctrl + C |
| Paste | Cmd/Ctrl + V |

## License

MIT
