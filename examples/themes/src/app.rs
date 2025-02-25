use egui::Vec2;
use egui_term::{
    ColorPalette, PtyEvent, TerminalBackend, TerminalTheme, TerminalView,
};
use std::sync::mpsc::Receiver;

pub struct App {
    terminal_backend: TerminalBackend,
    terminal_theme: TerminalTheme,
    pty_proxy_receiver: Receiver<(u64, egui_term::PtyEvent)>,
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

        Self {
            terminal_backend,
            terminal_theme: TerminalTheme::default(),
            pty_proxy_receiver,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok((_, PtyEvent::Exit)) = self.pty_proxy_receiver.try_recv() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("ubuntu").clicked() {
                    self.terminal_theme = egui_term::TerminalTheme::default();
                }

                if ui.button("3024 Day").clicked() {
                    self.terminal_theme =
                        egui_term::TerminalTheme::new(Box::new(ColorPalette {
                            background: String::from("#F7F7F7"),
                            foreground: String::from("#4A4543"),
                            black: String::from("#090300"),
                            red: String::from("#DB2D20"),
                            green: String::from("#01A252"),
                            yellow: String::from("#FDED02"),
                            blue: String::from("#01A0E4"),
                            magenta: String::from("#A16A94"),
                            cyan: String::from("#B5E4F4"),
                            white: String::from("#A5A2A2"),
                            bright_black: String::from("#5C5855"),
                            bright_red: String::from("#E8BBD0"),
                            bright_green: String::from("#3A3432"),
                            bright_yellow: String::from("#4A4543"),
                            bright_blue: String::from("#807D7C"),
                            bright_magenta: String::from("#D6D5D4"),
                            bright_cyan: String::from("#CDAB53"),
                            bright_white: String::from("#F7F7F7"),
                            ..Default::default()
                        }));
                }

                if ui.button("ubuntu").clicked() {
                    self.terminal_theme =
                        egui_term::TerminalTheme::new(Box::new(ColorPalette {
                            background: String::from("#300A24"),
                            foreground: String::from("#FFFFFF"),
                            black: String::from("#2E3436"),
                            red: String::from("#CC0000"),
                            green: String::from("#4E9A06"),
                            yellow: String::from("#C4A000"),
                            blue: String::from("#3465A4"),
                            magenta: String::from("#75507B"),
                            cyan: String::from("#06989A"),
                            white: String::from("#D3D7CF"),
                            bright_black: String::from("#555753"),
                            bright_red: String::from("#EF2929"),
                            bright_green: String::from("#8AE234"),
                            bright_yellow: String::from("#FCE94F"),
                            bright_blue: String::from("#729FCF"),
                            bright_magenta: String::from("#AD7FA8"),
                            bright_cyan: String::from("#34E2E2"),
                            bright_white: String::from("#EEEEEC"),
                            ..Default::default()
                        }));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(ui, &mut self.terminal_backend)
                .set_focus(true)
                .set_theme(self.terminal_theme.clone())
                .set_size(Vec2::new(
                    ui.available_width(),
                    ui.available_height(),
                ));

            ui.add(terminal);
        });
    }
}
