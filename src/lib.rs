mod theme;
mod backend;
mod font;
mod types;
mod bindings;

use alacritty_terminal::term::TermMode;
use alacritty_terminal::term::cell;
use alacritty_terminal::index::Point as TerminalGridPoint;
use backend::BackendCommand;
use bindings::{BindingAction, BindingsLayout, InputKind};
use egui::{Button, Id, PointerButton, PointerState};
use egui::Modifiers;
use egui::MouseWheelUnit;
use egui::Widget;
use egui::{Align2, Painter, Pos2, Rect, Response, Rounding, Stroke, Vec2};
use types::Size;

pub use font::TermFont;
pub use theme::TermTheme;
pub use backend::settings::BackendSettings;
pub use backend::TerminalBackend;
pub use alacritty_terminal::event::Event as BackendEvent;
use alacritty_terminal::selection::SelectionType;
use crate::backend::{LinkAction, MouseButton, MouseMode};

const EGUI_TERM_WIDGET_ID_PREFIX: &str = "egui_term::instance::";

#[derive(Debug)]
enum InputAction {
    BackendCall(BackendCommand),
    Ignore,
}

#[derive(Clone, Default, Debug)]
pub struct TerminalViewState {
    is_dragged: bool,
    is_focused: bool,
    scroll_pixels: f32,
    current_mouse_position_on_grid: TerminalGridPoint,
    keyboard_modifiers: Modifiers,
}

pub struct TerminalView<'a> {
    widget_id: Id,
    backend: &'a mut TerminalBackend,
    font: TermFont,
    theme: TermTheme,
    bindings_layout: BindingsLayout,
}

impl<'a> Widget for TerminalView<'a> {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        let (layout, painter) = ui.allocate_painter(
            ui.available_size(),
            egui::Sense::click(),
        );

        let widget_id = self.widget_id.clone();
        let mut state = ui.memory(
            |m| m.data.get_temp::<TerminalViewState>(widget_id)
                .unwrap_or_default()
        );

        self
            .focus(&layout)
            .resize(&layout)
            .process_input(&layout, &mut state)
            .show(&layout, &painter);

        ui.memory_mut(|m| m.data.insert_temp(widget_id, state));
        layout
    }
}

impl<'a> TerminalView<'a> {
    pub fn new(
        ui: &mut egui::Ui,
        backend: &'a mut TerminalBackend,
    ) -> Self {
        let widget_id = ui.make_persistent_id(
            format!("{}{}", EGUI_TERM_WIDGET_ID_PREFIX, backend.id),
        );

        Self {
            widget_id,
            backend,
            font: TermFont::default(),
            theme: TermTheme::default(),
            bindings_layout: BindingsLayout::new(),
        }
    }

    pub fn set_theme(mut self, theme: TermTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_font(mut self, font: TermFont) -> Self {
        self.font = font;
        self
    }

    fn focus(self, layout: &Response) -> Self {
        if layout.clicked() {
            layout.request_focus();
        }

        self
    }

    fn resize(self, layout: &Response) -> Self {
        self.backend.process_command(
            backend::BackendCommand::Resize(
                Size::from(layout.rect.size()),
                self.font.font_measure(&layout.ctx),
            )
        );

        self
    }

    fn process_input(self, layout: &Response, state: &mut TerminalViewState) -> Self {
        if !layout.has_focus() {
            return self;
        }

        layout.ctx.input(|i| {
            for event in &i.events {
                let input_action = match event {
                    egui::Event::Text(_) | egui::Event::Key { .. } => process_keyboard_event(
                        event,
                        &self.bindings_layout,
                        self.backend.last_content().terminal_mode,
                    ),
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        ..
                    } => process_mouse_wheel(state, self.font.font_type().size, unit, delta),
                    egui::Event::PointerButton {
                        button,
                        pressed,
                        modifiers,
                        pos,
                        ..
                    } => process_button_click(state, layout, self.backend, &i.pointer, button, pos, modifiers, pressed),
                    egui::Event::PointerMoved(pos) => process_mouse_move(state, layout, self.backend, pos, &i.modifiers),
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

    fn show(self, layout: &Response, painter: &Painter) {
        let content = self.backend.sync();
        let layout_offset = layout.rect.min;
        let font_size = self.font.font_measure(&layout.ctx);
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
    
            painter.rect(
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
                painter.text(
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

fn process_keyboard_event(
    event: &egui::Event,
    bindings_layout: &BindingsLayout,
    term_mode: TermMode,
) -> InputAction {    
    let mut action = InputAction::Ignore;
    match event {
        egui::Event::Text(text) => {
            action = InputAction::BackendCall(
                BackendCommand::Write(text.as_bytes().to_vec())
            );
        },
        egui::Event::Key {
            key,
            pressed,
            modifiers,
            ..
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
                _ => {},
            };
        }
        _ => {},
    }

    action
}

fn process_mouse_wheel(
    state: &mut TerminalViewState,
    font_size: f32,
    unit: &MouseWheelUnit,
    delta: &Vec2,
) -> InputAction {
    match unit {
        MouseWheelUnit::Line => {
            let lines = delta.y.signum() * delta.y.abs().ceil();
            InputAction::BackendCall(BackendCommand::Scroll(lines as i32))
        },
        MouseWheelUnit::Point => {
            state.scroll_pixels -= delta.y;
            let lines = (state.scroll_pixels / font_size).trunc();
            state.scroll_pixels %= font_size;
            if lines != 0.0 {
                InputAction::BackendCall(BackendCommand::Scroll(lines as i32))
            } else {
                InputAction::Ignore
            }
        },
        MouseWheelUnit::Page => InputAction::Ignore,
    }
}

fn process_button_click(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    pointer_state: &PointerState,
    button: &PointerButton,
    position: &Pos2,
    modifiers: &Modifiers,
    pressed: &bool,
) -> InputAction {
    match button {
        PointerButton::Primary => process_left_button(
            state,
            layout,
            backend,
            pointer_state,
            position,
            modifiers,
            pressed,
        ),
        _ => InputAction::Ignore
    }
}

fn process_left_button(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    pointer_state: &PointerState,
    position: &Pos2,
    modifiers: &Modifiers,
    pressed: &bool,
) -> InputAction {
    let action = if *pressed {
        process_left_button_pressed(
            state,
            layout,
            backend,
            pointer_state,
            position,
            modifiers,
        )
    } else {
        process_left_button_released(state, backend, modifiers)
    };

    action
}

fn process_left_button_pressed(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    pointer_state: &PointerState,
    position: &Pos2,
    modifiers: &Modifiers
) -> InputAction {
    let terminal_mode = backend.last_content().terminal_mode;
    let action = if terminal_mode.contains(TermMode::SGR_MOUSE) && modifiers.is_none() {
        InputAction::BackendCall(
            BackendCommand::MouseReport(
                MouseMode::Sgr,
                MouseButton::LeftButton,
                state.current_mouse_position_on_grid,
                true
            )
        )
    } else {
        let selection_type = if pointer_state.button_double_clicked(PointerButton::Primary) {
            SelectionType::Semantic
        } else if pointer_state.button_triple_clicked(PointerButton::Primary) {
            SelectionType::Lines
        } else {
            SelectionType::Simple
        };

        InputAction::BackendCall(
            BackendCommand::SelectStart(
                selection_type,
                (
                    position.x - layout.rect.min.x,
                    position.y - layout.rect.min.y,
                )
            )
        )
    };

    state.is_dragged = true;
    action
}

fn process_left_button_released(
    state: &mut TerminalViewState,
    backend: &TerminalBackend,
    modifiers: &Modifiers
) -> InputAction {
    state.is_dragged = false;
    let terminal_content = backend.last_content();
    if terminal_content.terminal_mode.contains(TermMode::SGR_MOUSE) {
        return InputAction::BackendCall(
            BackendCommand::MouseReport(
                MouseMode::Sgr,
                MouseButton::LeftButton,
                state.current_mouse_position_on_grid,
                false,
            ),
        );
    }

    // if bindings.get_action(
    //     InputKind::Mouse(iced_core::mouse::Button::Left),
    //     state.keyboard_modifiers,
    //     *terminal_mode,
    // ) == BindingAction::LinkOpen
    // {
    //     commands.push(Command::ProcessBackendCommand(
    //         BackendCommand::ProcessLink(
    //             LinkAction::Open,
    //             state.mouse_position_on_grid,
    //         ),
    //     ));
    // }

    InputAction::Ignore
}

fn process_mouse_move(
    state: &mut TerminalViewState,
    layout: &Response,
    backend: &TerminalBackend,
    position: &Pos2,
    modifiers: &Modifiers
) -> InputAction {
    let terminal_content = backend.last_content();
    let cursor_x = position.x - layout.rect.min.x;
    let cursor_y = position.y - layout.rect.min.y;
    state.current_mouse_position_on_grid = TerminalBackend::selection_point(
        cursor_x,
        cursor_y,
        &terminal_content.terminal_size,
        terminal_content.grid.display_offset(),
    );

    println!("{} {} {:?}", position.x, position.y, state);

    // Handle command or selection update based on terminal mode and modifiers
    if state.is_dragged {
        let terminal_mode = terminal_content.terminal_mode;
        let cmd = if terminal_mode.contains(TermMode::SGR_MOUSE)
            && modifiers.is_none()
        {
            InputAction::BackendCall(
                    BackendCommand::MouseReport(
                    MouseMode::Sgr,
                    MouseButton::LeftMove,
                    state.current_mouse_position_on_grid,
                    true,
                )
            )
        } else {
            InputAction::BackendCall(
                BackendCommand::SelectUpdate((
                    cursor_x, cursor_y,
                ))
            )
        };

        return cmd
    }

    // Handle link hover if applicable
    if let Modifiers::COMMAND = *modifiers {
        return InputAction::BackendCall(
            BackendCommand::ProcessLink(
                LinkAction::Hover,
                state.current_mouse_position_on_grid,
            ),
        )
    }

    InputAction::Ignore
}