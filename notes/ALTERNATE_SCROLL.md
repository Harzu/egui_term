# ALTERNATE_SCROLL Fix

## Problem

Mouse wheel scrolling was not working correctly in applications using ncurses with alternate screen mode (e.g., opencode, tmux, less, vim). Instead of scrolling the content within the application, the terminal was sending arrow key sequences (ESC O A / ESC O B).

## Root Cause

The issue was in the scroll handling logic in `src/view.rs` and `src/backend/mod.rs`:

1. **Incorrect priority check**: The condition `terminal.mode().contains(TermMode::ALTERNATE_SCROLL | TermMode::ALT_SCREEN)` would return `true` even when `MOUSE_MODE` was active, because `contains()` checks if ANY of the flags are present.

2. **Missing terminal mode priority**: When `MOUSE_MODE` (specifically `SGR_MOUSE`) is active in ncurses applications, the application expects mouse wheel events via SGR protocol, not arrow key sequences.

3. **Focus/pointer check issue**: All input events were blocked if either focus was lost OR pointer was outside widget, which prevented scroll from working even when the mouse was over the widget.

## Solution

### 1. Corrected terminal mode priority

Updated scroll handling in `src/view.rs` to follow alacrity's priority order:

```rust
// Priority 1: MOUSE_MODE (SGR_MOUSE active) → Send mouse wheel report
if terminal_mode.intersects(TermMode::MOUSE_MODE) {
    // Send SGR mouse wheel report
}
// Priority 2: ALT_SCREEN | ALTERNATE_SCROLL → Send arrow keys
else if terminal_mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) {
    // Send ESC O A / ESC O B
}
// Priority 3: Default → Scroll viewport
else {
    // Scroll terminal viewport
}
```

Key changes:
- Use `intersects()` for `MOUSE_MODE` check (it's a combination of multiple flags)
- Ensure MOUSE_MODE is checked BEFORE ALTERNATE_SCROLL
- Use `contains()` for ALT_SCREEN | ALTERNATE_SCROLL combination

### 2. Separated keyboard and mouse event handling

Modified `process_input()` in `src/view.rs` to have different focus requirements:

- **Keyboard events** (Text, Key, Copy, Paste): Require `layout.has_focus()`
- **Mouse events** (MouseWheel, PointerButton, PointerMoved): Require `layout.contains_pointer()`

This allows mouse wheel to work even when keyboard focus is on different widgets.

### 3. Inverted scroll direction

Adjusted scroll direction to match standard behavior:

- **Mouse wheel up** → Scroll content up (send ScrollUp / ESC O A)
- **Mouse wheel down** → Scroll content down (send ScrollDown / ESC O B)

## Files Changed

- `src/view.rs`: 
  - Modified `process_input()` to separate keyboard/mouse event handling
  - Rewrote `process_mouse_wheel()` with correct terminal mode priority
  - Removed rigid focus check at the start

- `src/backend/mod.rs`:
  - Simplified `scroll()` function to only handle viewport scrolling

## Testing

Tested with opencode (ncurses-based application) in alternate screen mode:
- ✅ Mouse wheel now correctly scrolls application content
- ✅ Mouse wheel events are sent via SGR protocol when MOUSE_MODE is active
- ✅ Scroll direction matches standard behavior
- ✅ Works without requiring explicit focus

## References

- Alacrity scroll handling: `alacritty/src/input/mod.rs` - `scroll_terminal()` function
- Terminal mode definitions: `alacritty_terminal/src/term/mod.rs` - `TermMode` enum
- DEC Private Mode 1007 (ALTERNATE_SCROLL): Controls arrow key substitution for scroll events
