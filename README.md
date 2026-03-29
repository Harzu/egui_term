# egui_term fork gives you:

## Bug Fixes

- **Copying multi-line text** — added newline characters when copying terminal text
- **Bracketed paste** — fixed proper bracketed paste support
- **Ctrl+X** — fixed handling (was intercepted by egui)
- **Kill child process** — proper termination when TerminalBackend is dropped
- **Alternate screen scroll** — fixed scroll handling in MOUSE_MODE

## New Features

- **Ctrl+Shift+PageUp/Down** — bindings for terminal scrolling
- **ScrollPageUp/ScrollPageDown** — page scrolling support
- **Autoscrolling** — automatic scrolling while input
- **UP/DOWN** — navigation key improvements
- **clear_history()** — method to clear terminal history
- **scroll_to_bottom()** — method to scroll to bottom
- **Shell arguments** — support for shell arguments (--login for bash)
- **ENV support** — TERM and COLORTERM in BackendSettings

---

Forked from [Harzu/egui_term](https://github.com/Harzu/egui_term)
