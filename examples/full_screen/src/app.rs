use egui::Vec2;
use egui_term::{TerminalBackend, TerminalView};

pub struct App {
    terminal_backend_1: TerminalBackend,
    terminal_backend_2: TerminalBackend,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let terminal_backend_1 = TerminalBackend::new(
            0,
            cc.egui_ctx.clone(),
            egui_term::BackendSettings::default(),
        ).unwrap();

        let terminal_backend_2 = TerminalBackend::new(
            1,
            cc.egui_ctx.clone(),
            egui_term::BackendSettings::default(),
        ).unwrap();

        Self {
            terminal_backend_1,
            terminal_backend_2
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal_1 = TerminalView::new(
                ui,
                &mut self.terminal_backend_1,
            )
                .set_focus(true)
                .set_size(Vec2::new(ui.available_width(), ui.available_height()));

            // let terminal_2 = TerminalView::new(
            //     ui,
            //     &mut self.terminal_backend_2,
            // )
            //     .set_focus(false)
            //     .set_size(Vec2::new(ui.available_width(), ui.available_height() / 2.0));

            ui.add(terminal_1);
            // ui.add(terminal_2);
        });
    }
}
