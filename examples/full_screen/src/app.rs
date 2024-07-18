use egui_term::{TerminalBackend, TerminalView};
use std::{sync::mpsc, thread::JoinHandle};

pub struct App {
    terminal_backend: TerminalBackend,
    _event_listener: JoinHandle<()>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let terminal_backend = TerminalBackend::new(
            0,
            event_tx,
            egui_term::BackendSettings::default(),
        ).unwrap();

        let ctx = cc.egui_ctx.clone();
        let event_listener = std::thread::Builder::new()
            .name("backend_event_listener".into())
            .spawn(move || {
                loop {
                    if let Ok(event) = event_rx.recv() {
                        match event {
                            egui_term::BackendEvent::Exit => {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                break;
                            },
                            egui_term::BackendEvent::Wakeup => {
                                ctx.clone().request_repaint();
                            },
                            _ => {},
                        }
                    }
                }
            }).unwrap();

        Self {
            terminal_backend,
            _event_listener: event_listener,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Sense::click(),
            );

            TerminalView::new(
                &mut self.terminal_backend,
                response,
                painter,
            )
                .resize_handler()
                .input_handler()
                .has_focus(true)
                .show()
        });
    }
}
