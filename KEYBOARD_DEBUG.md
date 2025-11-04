# Keyboard Debugging Guide

## Debug Mode Enabled

The TUI now shows what keys your terminal is sending. When you run `zlyph`, you'll see debug output like:

```
Key: Left, Mods: SUPER       # Cmd+Left
Key: Right, Mods: ALT        # Alt+Right
Key: Char('b'), Mods: ALT    # Terminal intercepted Alt+Left as Escape+b
```

## How to Test

1. **Run zlyph with debug output visible**:
   ```bash
   zlyph 2>debug.log &
   tail -f debug.log
   ```

2. **Or run in one terminal and watch stderr**:
   ```bash
   zlyph
   # Debug output appears in the terminal
   ```

3. **Test each problematic key**:
   - Press `Cmd+Left` - should see: `Key: Left, Mods: SUPER`
   - Press `Cmd+Right` - should see: `Key: Right, Mods: SUPER`
   - Press `Alt+Left` - should see: `Key: Left, Mods: ALT` (if configured)
   - Press `Alt+Right` - should see: `Key: Right, Mods: ALT` (if configured)

## What You Might See

### ✅ Working (Terminal Configured):
```
Key: Left, Mods: SUPER        # Cmd+Left works
Key: Right, Mods: SUPER       # Cmd+Right works  
Key: Left, Mods: ALT          # Alt+Left works
Key: Right, Mods: ALT         # Alt+Right works
```

### ❌ Not Working (Terminal Not Configured):
```
Key: Left, Mods: SUPER        # Cmd+Left works
Key: Right, Mods: SUPER       # Cmd+Right works
Key: Char('b'), Mods: ALT     # Alt+Left intercepted by terminal
Key: Char('f'), Mods: ALT     # Alt+Right intercepted by terminal
```

If you see `Char('b')` or `Char('f')` instead of arrow keys, your terminal needs configuration.

## All Delete Combinations Now Work

Once keys are passing through correctly, these all work:

| Shortcut | Action |
|----------|--------|
| `Backspace` | Delete char left |
| `Delete` | Delete char right |
| `Ctrl+Backspace` | Delete to beginning of line |
| `Ctrl+Delete` | Delete to end of line |
| `Alt+Backspace` | Delete word left |
| `Alt+Delete` | Delete word right |
| `Cmd+Backspace` | Delete entire line |
| `Cmd+Delete` | Delete to end of line |

## Terminal Configuration

### Terminal.app
1. Preferences → Profiles → Keyboard
2. Check "Use Option as Meta key"

### iTerm2
1. Preferences → Profiles → Keys  
2. Set "Left Option key" to "Esc+"

### Test After Config
1. Close and reopen terminal
2. Run `zlyph` again
3. Press `Alt+Left` - should now see `Key: Left, Mods: ALT`

## Share Debug Output

If keys still don't work, run:
```bash
zlyph 2>&1 | tee keyboard-debug.txt
# Press the problematic keys
# Ctrl+W to quit
# Share keyboard-debug.txt
```
