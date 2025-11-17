<!-- markdownlint-disable MD041 MD033 MD045 -->
<div align="center">

# egui_term

[![GitHub License][license-badge]][license-link]
[![crates.io][crates.io-badge]][crates.io-link]
[![docs.rs][docs.rs-badge]][docs.rs-link]
[![Rust CI][ci-badge]][ci-link]

Terminal emulator widget powered by EGUI framework and alacritty terminal backend.

<a href="./examples/full_screen">
  <img src="examples/full_screen/assets/screenshot.png" width="275px">
</a>
<a href="./examples/tabs">
  <img src="examples/tabs/assets/screenshot.png" width="273px">
</a>

</div>

## Features

The widget is currently under development and does not provide full terminal features make sure that widget is covered everything you want.

- PTY content rendering
- Multiple instance support
- Basic keyboard input
- Adding custom keyboard or mouse bindings
- Resizing
- Scrolling
- Focusing
- Selecting
- Changing Font/Color scheme
- Hyperlinks processing (hover/open)

This widget was tested on MacOS, Linux, and Windows.

## Examples

You can also look at [examples](./examples) directory for more information about widget using.

- [full_screen](./examples/full_screen/) - The basic example of terminal emulator.
- [tabs](./examples/tabs/) - The example with tab widget that show how multiple instance feature work.
- [custom_bindings](./examples/custom_bindings/) - The example that show how you can add custom keyboard or mouse bindings to your terminal emulator app.
- [themes](./examples/themes/) - The example that show how you can change terminal color scheme.
- [fonts](./examples/fonts/) - The examples that show how you can change font type or font size in your terminal emulator app.

## Dependencies

[![dependency status][deps.rs-badge]][deps.rs-link]

- [alacritty_terminal](https://github.com/alacritty/alacritty) (Apache-2.0)
- [anyhow](https://github.com/dtolnay/anyhow) (MIT OR Apache-2.0)
- [egui](https://github.com/emilk/egui) (MIT OR Apache-2.0)
- [open](https://github.com/Byron/open-rs) (MIT)

[license-badge]: https://img.shields.io/github/license/Harzu/egui_term
[license-link]: https://github.com/Harzu/egui_term/blob/main/LICENSE
[crates.io-badge]: https://img.shields.io/crates/v/egui_term
[crates.io-link]: https://crates.io/crates/egui_term
[docs.rs-badge]: https://img.shields.io/docsrs/egui_term
[docs.rs-link]: https://docs.rs/egui_term
[ci-badge]: https://github.com/Harzu/egui_term/actions/workflows/rust.yml/badge.svg
[ci-link]: https://github.com/Harzu/egui_term/actions/workflows/rust.yml
[deps.rs-badge]: https://img.shields.io/deps-rs/egui_term/latest?style=for-the-badge&label=dependency%20status
[deps.rs-link]: https://deps.rs/crate/egui_term
