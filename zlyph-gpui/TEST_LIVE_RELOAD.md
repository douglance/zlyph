# Testing Live Reload

Both editors now automatically reload when the file changes externally.

## Test Setup

1. **Terminal 1**: Open GPUI editor
   ```bash
   cargo run -p zlyph-gpui
   ```

2. **Terminal 2**: Modify file directly
   ```bash
   echo "Hello from terminal!" >> ~/.config/zlyph/default.txt
   ```

3. **Verify**: Check GPUI window - should show new content immediately

## Test Scenario 2

1. **Terminal 1**: Open TUI
   ```bash
   cargo run -p zlyph-tui
   ```

2. **Terminal 2**: Open GPUI at the same time
   ```bash
   cargo run -p zlyph-gpui
   ```

3. **Type in TUI**: Changes appear in GPUI in real-time (within 100ms)

4. **Type in GPUI**: Changes appear in TUI on next render

## Implementation Details

### TUI
- Polls every 100ms for file changes
- Checks modification time
- Reloads automatically when file changes externally

### GPUI
- Checks on every render frame (typically 60 FPS)
- Compares modification time
- Reloads and re-renders automatically

### Conflict Resolution
- Both editors save after every keystroke
- Last write wins (standard file semantics)
- No locking needed - OS handles atomic writes

## Expected Behavior

✅ Edit in TUI → See changes in GPUI immediately  
✅ Edit in GPUI → See changes in TUI immediately  
✅ Edit file directly → Both editors reload automatically  
✅ No data loss (auto-save after every action)  
✅ Undo/redo preserved within each editor session  
