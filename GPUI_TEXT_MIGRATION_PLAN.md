# GPUI Text Component Migration Plan

## Current Implementation Problems

### 1. **Approximated Character Width**
```rust
let char_width = font_size_px * 0.6;  // WRONG - assumes monospace average
```
- Doesn't work for proportional fonts
- Incorrect for Unicode/emoji
- No ligature support
- No kerning consideration

### 2. **Manual Text Rendering**
```rust
div().child(line.to_string())  // Basic string rendering
```
- No text shaping
- No proper text layout
- No accurate measurement
- No glyph positioning

### 3. **Coordinate Math Issues**
- Mouse click positioning is approximate
- Cursor rendering uses estimated positions
- Selection rendering inaccurate for non-ASCII
- Wrapping calculations would be complex

## Proposed Solution: Use GPUI Text Components

### Phase 1: Research GPUI Text APIs

**Investigate:**
1. `gpui::text` module
2. `StyledText` component
3. `TextLayout` and `TextRun` types
4. Text measurement APIs (`measure_text`, `shaped_line`)
5. Text element builders

**Check Zed source:**
- `editor/src/element.rs` - How Zed renders editor text
- Text shaping and layout logic
- Cursor/selection rendering
- Mouse-to-text-position conversion

### Phase 2: Refactor Text Storage

**Current:**
```rust
content: String              // Single string
cursor_position: usize       // Byte offset
```

**Proposed:**
```rust
buffer: TextBuffer           // Rope data structure
cursor: BufferPosition       // (line, column)
viewport: ViewportState      // Scroll position
text_layout: Option<Layout>  // Cached layout
```

**Benefits:**
- Efficient editing (rope vs string)
- Natural line/column addressing
- Proper layout caching

### Phase 3: Implement Text Rendering with GPUI

**Replace manual rendering with GPUI text components:**

```rust
// Current (WRONG)
div().child(line.to_string())

// Proposed
StyledText::new(line)
    .with_font(font)
    .with_size(font_size)
    .with_color(text_color)
    .with_highlights(selection_ranges)
```

**Key changes:**
1. Use `ShapedLine` for proper text shaping
2. Use `LineLayout` for wrapping calculations
3. Use text measurement for accurate positioning
4. Cache layouts for performance

### Phase 4: Fix Coordinate Calculations

**Mouse Position → Text Position:**
```rust
// Current (approximation)
fn position_from_mouse(&self, pos: Point) -> usize {
    let char_width = font_size * 0.6;  // WRONG
    let col = (pos.x / char_width) as usize;
    // ...
}

// Proposed (accurate)
fn position_from_mouse(&self, pos: Point, cx: &Context) -> BufferPosition {
    let layout = self.get_or_compute_layout(cx);
    layout.position_for_point(pos)  // Uses actual glyph positions
}
```

**Text Position → Screen Coordinates:**
```rust
// For cursor rendering
fn screen_position(&self, pos: BufferPosition, cx: &Context) -> Point {
    let layout = self.get_or_compute_layout(cx);
    layout.point_for_position(pos)
}
```

### Phase 5: Implement Text Wrapping

**With proper text layout:**
```rust
struct LineLayout {
    visual_lines: Vec<VisualLine>,  // Wrapped segments
    width: Pixels,
}

struct VisualLine {
    text_range: Range<usize>,  // Byte range in source
    glyphs: Vec<Glyph>,        // Shaped glyphs
    width: Pixels,
}
```

**Wrapping algorithm:**
1. Measure each word
2. If word exceeds line width, break with hyphen
3. Cache visual line breaks
4. Map cursor position → visual line + column

### Phase 6: Implement Hyphenation

**Add hyphenation library:**
```toml
[dependencies]
hyphenation = "0.8"
```

**Usage:**
```rust
fn wrap_with_hyphenation(word: &str, available_width: Pixels) -> Vec<String> {
    let dictionary = hyphenation::Load::en_us();
    let breaks = dictionary.hyphenate(word);
    // Split at break points that fit width
}
```

## Implementation Checklist

### Core Text Infrastructure
- [ ] Research GPUI text APIs in Zed source
- [ ] Implement `TextBuffer` with rope data structure
- [ ] Add `BufferPosition` type (line, column)
- [ ] Implement text layout caching

### Rendering
- [ ] Replace `div().child(string)` with `StyledText`
- [ ] Implement proper text shaping
- [ ] Cache shaped text layouts
- [ ] Add syntax highlighting support (future)

### Coordinate Mapping
- [ ] `mouse_position → buffer_position` with text layout
- [ ] `buffer_position → screen_coordinates` for cursor
- [ ] `buffer_range → screen_rects` for selection
- [ ] Handle multi-line selections

### Text Wrapping
- [ ] Implement word wrapping with measurement
- [ ] Add soft wrap at word boundaries
- [ ] Implement hyphenation for long words
- [ ] Cache wrap points per line

### Navigation
- [ ] Fix arrow keys with wrapped lines
- [ ] Fix mouse click with wrapped lines
- [ ] Fix selection with wrapped lines
- [ ] Home/End respect visual vs logical lines

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_mouse_to_position_ascii() {
    // Test basic ASCII text
}

#[test]
fn test_mouse_to_position_unicode() {
    // Test emoji, combining characters
}

#[test]
fn test_wrapping_boundary() {
    // Test wrap at exactly line width
}

#[test]
fn test_hyphenation() {
    // Test long word hyphenation
}
```

### Integration Tests
- Click at various positions
- Drag to select across wraps
- Type at wrap boundaries
- Navigate with arrows across wraps

## Performance Considerations

### Optimization Strategies
1. **Lazy layout computation** - Only layout visible lines
2. **Layout caching** - Cache until text changes
3. **Incremental updates** - Only re-layout affected lines
4. **Virtual scrolling** - Don't render off-screen lines

### Metrics to Track
- Layout computation time
- Re-layout frequency
- Memory usage (cached layouts)
- Frame rate during scrolling

## Alternative: Disable Wrapping (Current Choice)

**Pros:**
- Simple, proven approach (VS Code, Vim)
- Accurate coordinate math
- Better for code editing
- Less complexity

**Cons:**
- Long lines extend off-screen
- Horizontal scrolling needed
- Less suitable for prose

**Current Decision:** Disabled wrapping for simplicity and accuracy. This matches standard code editor behavior.

## Future Work

After basic editor is stable:
1. Implement GPUI text components migration
2. Add optional soft-wrap mode
3. Implement hyphenation
4. Add syntax highlighting
5. Support variable-width fonts

## References

- GPUI text module source code
- Zed editor element.rs
- Text shaping algorithms (HarfBuzz)
- Hyphenation patterns (TeX)
- Rope data structures (Xi editor)

---

**Status:** Deferred - Using no-wrap approach for MVP
**Priority:** P2 - Nice to have for future
**Estimated Effort:** 2-3 weeks full implementation
