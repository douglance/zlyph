# Terminal Configuration for Zlyph

## Mac Terminal Setup

If Alt+arrow keys aren't working for word jumping, you need to configure your terminal:

### Terminal.app (Mac default)

1. Open Terminal preferences (`Cmd+,`)
2. Go to "Profiles" tab
3. Select your profile
4. Click "Keyboard" tab
5. Check "Use Option as Meta key"

### iTerm2

1. Open iTerm2 preferences (`Cmd+,`)
2. Go to "Profiles" → "Keys"
3. Set "Left Option key" to "Esc+"
4. Set "Right Option key" to "Esc+" (or keep as normal for special characters)

### Alternative: Use Ctrl instead

If you prefer not to change terminal settings, use these shortcuts instead:

| Shortcut | Action |
|----------|--------|
| `Ctrl+Left/Right` | Word jump (if your terminal supports it) |
| `Cmd+Left/Right` | Line start/end ✅ (works without config) |
| `Cmd+Backspace` | Delete line ✅ (works without config) |

## Verified Working Shortcuts

These work without any terminal configuration:

- `Cmd+Left` - Jump to line start
- `Cmd+Right` - Jump to end of line  
- `Cmd+Backspace` - Delete current line
- `Ctrl+Z` - Undo
- `Ctrl+Shift+Z` - Redo
- `Ctrl+A` - Select all
- `Shift+Arrows` - Text selection
- `Ctrl+W` - Quit

## Testing Alt Keys

After configuring your terminal, test with:

```bash
zlyph
# Type: "The quick brown fox jumps"
# Press Alt+Right repeatedly - should jump by word
# Press Alt+Left - should jump back by word
```

If Alt+arrow keys still don't work, your terminal may not support them. Use Cmd+Left/Right for line navigation instead.
