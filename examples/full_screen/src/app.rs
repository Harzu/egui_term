use egui_term::{TerminalBackend, TerminalView};

pub struct App {
    terminal_backend: TerminalBackend,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let terminal_backend = TerminalBackend::new(
            0,
            cc.egui_ctx.clone(),
            egui_term::BackendSettings::default(),
        ).unwrap();

        Self {
            terminal_backend,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(
                ui,
                &mut self.terminal_backend,
            );

            ui.add(terminal);
        });
    }
}
