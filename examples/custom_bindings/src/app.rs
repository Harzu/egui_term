use egui::{Key, Modifiers, Vec2};
use egui_term::{
    generate_bindings, Binding, BindingAction, InputKind, KeyboardBinding,
    PtyEvent, TerminalBackend, TerminalMode, TerminalView,
};
use std::sync::mpsc::Receiver;

pub struct App {
    terminal_backend: TerminalBackend,
    pty_proxy_receiver: Receiver<(u64, egui_term::PtyEvent)>,
    custom_terminal_bindings: Vec<(Binding<InputKind>, BindingAction)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let system_shell = std::env::var("SHELL")
            .expect("SHELL variable is not defined")
            .to_string();

        let (pty_proxy_sender, pty_proxy_receiver) = std::sync::mpsc::channel();
        let terminal_backend = TerminalBackend::new(
            0,
            cc.egui_ctx.clone(),
            pty_proxy_sender.clone(),
            egui_term::BackendSettings {
                shell: system_shell,
                args: vec![],
            },
        )
        .unwrap();

        let mut custom_terminal_bindings = vec![
            (
                Binding {
                    target: InputKind::KeyCode(egui::Key::C),
                    modifiers: Modifiers::SHIFT,
                    terminal_mode_include: TerminalMode::ALT_SCREEN,
                    terminal_mode_exclude: TerminalMode::empty(),
                },
                BindingAction::Paste,
            ),
            (
                Binding {
                    target: InputKind::KeyCode(egui::Key::A),
                    modifiers: Modifiers::SHIFT | Modifiers::CTRL,
                    terminal_mode_include: TerminalMode::empty(),
                    terminal_mode_exclude: TerminalMode::empty(),
                },
                BindingAction::Char('B'),
            ),
            (
                Binding {
                    target: InputKind::KeyCode(egui::Key::B),
                    modifiers: Modifiers::SHIFT | Modifiers::CTRL,
                    terminal_mode_include: TerminalMode::empty(),
                    terminal_mode_exclude: TerminalMode::empty(),
                },
                BindingAction::Esc("\x1b[5~".into()),
            ),
        ];

        custom_terminal_bindings = [
            custom_terminal_bindings,
            // You can also use generate_bindings macros
            generate_bindings!(
                KeyboardBinding;
                L, Modifiers::SHIFT; BindingAction::Char('K');
            ),
        ]
        .concat();

        Self {
            terminal_backend,
            pty_proxy_receiver,
            custom_terminal_bindings,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok((_, PtyEvent::Exit)) = self.pty_proxy_receiver.try_recv() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(ui, &mut self.terminal_backend)
                .set_focus(true)
                .add_bindings(self.custom_terminal_bindings.clone())
                .set_size(Vec2::new(
                    ui.available_width(),
                    ui.available_height(),
                ));

            ui.add(terminal);
        });
    }
}
