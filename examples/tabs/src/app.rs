use std::{collections::HashMap, sync::mpsc::{self, Receiver, Sender}, thread::JoinHandle};
use egui_term::{TerminalBackend, TerminalView};

#[derive(Debug, Clone)]
pub enum Command {
    BackendEventReceived(u64, egui_term::BackendEvent)
}

pub struct App {
    command_sender: Sender<Command>,
    command_receiver: Receiver<Command>,
    active_tab: u64,
    terminal_tabs: HashMap<u64, (TerminalBackend, JoinHandle<()>)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        Self {
            command_sender,
            command_receiver,
            active_tab: 0,
            terminal_tabs: HashMap::new()
        }
    }

    fn create_tab(&mut self, ctx: egui::Context) {
        let new_tab_id = self.terminal_tabs.len() as u64;
        let (event_tx, event_rx) = mpsc::channel();
        let terminal_backend = TerminalBackend::new(
            new_tab_id,
            event_tx,
            egui_term::BackendSettings::default(),
        ).unwrap();

        let sender = self.command_sender.clone();
        let event_listener = std::thread::Builder::new()
            .name(format!("backend_event_listener_{}", new_tab_id))
            .spawn(move || {
                loop {
                    if let Ok(e) = event_rx.recv() {
                        sender.send(Command::BackendEventReceived(new_tab_id, e)).unwrap();
                        ctx.clone().request_repaint();
                    }
                }
            }).unwrap();

        let _ = self.terminal_tabs.insert(new_tab_id, (terminal_backend, event_listener));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for (i, tab) in self.terminal_tabs.iter().enumerate() {
                    if ui.button(format!("{}", tab.0)).clicked() {
                        self.active_tab = tab.0.clone();
                        println!("Switched to {}", tab.0);
                    }
                }

                if ui.button("+").clicked() {
                    self.create_tab(ctx.clone());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Sense::click(),
            );

            if let Some(backend) = self.terminal_tabs.get_mut(&self.active_tab) {
                TerminalView::new(
                    &mut backend.0,
                    response,
                    painter,
                )
                    .resize_handler()
                    .input_handler()
                    .has_focus(true)
                    .show()
            }
        });
    }
}
