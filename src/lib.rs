mod actions;
mod theme;
mod backend;
mod font;
mod types;
mod bindings;

use alacritty_terminal::term::TermMode;
use alacritty_terminal::term::cell;
use backend::BackendCommand;
use bindings::{BindingAction, BindingsLayout, InputKind};
use egui::{Align2, Painter, Pos2, Rect, Response, Rounding, Stroke, Vec2};
use types::Size;

pub use font::TermFont;
pub use theme::TermTheme;
pub use backend::settings::BackendSettings;
pub use backend::TerminalBackend;
pub use alacritty_terminal::event::Event as BackendEvent;

pub struct TerminalView<'a> {
    backend: &'a mut TerminalBackend,
    layout: Response,
    painter: Painter,
    font: TermFont,
    theme: TermTheme,
    bindings_layout: BindingsLayout,
}

impl<'a> TerminalView<'a> {
    pub fn new(
        backend: &'a mut TerminalBackend,
        layout: Response,
        painter: Painter,
    ) -> Self {
        Self {
            backend,
            layout,
            painter,
            font: TermFont::default(),
            theme: TermTheme::default(),
            bindings_layout: BindingsLayout::new(),
        }
    }

    pub fn has_focus(self, enable: bool) -> Self {
        if enable {
            self.layout.request_focus();
        } else {
            self.layout.surrender_focus();
        }

        self
    }

    pub fn set_theme(mut self, theme: TermTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_font(mut self, font: TermFont) -> Self {
        self.font = font;
        self
    }

    pub fn resize_handler(self) -> Self {
        self.backend.process_command(
            backend::BackendCommand::Resize(
                Size::from(self.layout.rect.size()),
                self.font.font_measure(&self.layout.ctx),
            )
        );

        self
    }

    pub fn input_handler(self) -> Self {
        if !self.layout.has_focus() {
            return self;
        }

        self.layout.ctx.input(|i| {
            for event in &i.events {
                let input_action = match event {
                    egui::Event::Text(_) | egui::Event::Key { .. } => handle_keyboard_event(
                        event,
                        &self.bindings_layout,
                        self.backend.last_content().terminal_mode,
                    ),
                    _ => InputAction::Ignore,
                };

                match input_action {
                    InputAction::BackendCall(cmd) => {
                        self.backend.process_command(cmd);
                    },
                    InputAction::Ignore => {},
                }
            }
        });

        self
    }

    pub fn show(self) {
        let content = self.backend.sync();
        let layout_offset = self.layout.rect.min;
        let font_size = self.font.font_measure(&self.layout.ctx);
        for indexed in content.grid.display_iter() {
            let x = layout_offset.x
                + (indexed.point.column.0 as f32 * font_size.width);
            let y = layout_offset.y
                + ((indexed.point.line.0 as f32
                    + content.grid.display_offset() as f32)
                    * font_size.height);
    
            let mut fg = self.theme.get_color(indexed.fg);
            let mut bg = self.theme.get_color(indexed.bg);
    
            if indexed.cell.flags.contains(cell::Flags::INVERSE)
                || content
                    .selectable_range
                    .map_or(false, |r| r.contains(indexed.point))
            {
                std::mem::swap(&mut fg, &mut bg);
            }
    
            self.painter.rect(
                Rect::from_min_size(
                    Pos2::new(x, y), 
                    Vec2::new(font_size.width, font_size.height),
                ),
                Rounding::default(),
                bg, 
                Stroke::NONE
            );
    
            if indexed.c != ' ' && indexed.c != '\t' {
                let pos = Pos2 {
                        x: x + (font_size.width / 2.0),
                        y: y + (font_size.height / 2.0),
                };
                self.painter.text(
                    pos, 
                    Align2::CENTER_CENTER, 
                    indexed.c, 
                    self.font.font_type(),
                    fg,
                );
            }
        }
    }
}

enum InputAction {
    BackendCall(BackendCommand),
    Ignore,
}

fn handle_keyboard_event(
    event: &egui::Event,
    bindings_layout: &BindingsLayout,
    term_mode: TermMode,
) -> InputAction {    
    let mut action = InputAction::Ignore;

    match event {
        egui::Event::Text(c) => {
            action = InputAction::BackendCall(BackendCommand::Write(c.as_bytes().to_vec()))
        },
        egui::Event::Key {
            key,
            physical_key,
            pressed,
            repeat,
            modifiers
        } => {
            if !pressed {
                return action;
            }

            let binding_action = bindings_layout.get_action(
                InputKind::KeyCode(*key),
                *modifiers,
                term_mode,
            );

            match binding_action {
                BindingAction::Char(c) => {
                    let mut buf = [0, 0, 0, 0];
                    let str = c.encode_utf8(&mut buf);
                    action = InputAction::BackendCall(
                        BackendCommand::Write(str.as_bytes().to_vec()),
                    );
                },
                BindingAction::Esc(seq) => {
                    action = InputAction::BackendCall(
                        BackendCommand::Write(seq.as_bytes().to_vec()),
                    );
                },
                // BindingAction::Paste => {
                //     if let Some(data) = clipboard.read(ClipboardKind::Standard)
                //     {
                //         let input: Vec<u8> = data.bytes().collect();
                //         return Some(Command::ProcessBackendCommand(
                //             BackendCommand::Write(input),
                //         ));
                //     }
                // },
                // BindingAction::Copy => {
                //     clipboard.write(
                //         ClipboardKind::Standard,
                //         backend.selectable_content(),
                //     );
                // },
                _ => {},
            };
        }
        _ => {},
    }

    action
}