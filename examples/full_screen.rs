#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    eframe::run_native(
        "full_screen_example",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

use egui::Vec2;
use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::sync::mpsc::Receiver;

pub struct App {
    terminal_backend: TerminalBackend,
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
            },
        )
        .unwrap();

        Self {
            terminal_backend,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(ui, &mut self.terminal_backend)
                .set_focus(true)
                .set_size(Vec2::new(
                    ui.available_width(),
                    ui.available_height(),
                ));

            ui.add(terminal);
        });
    }
}
