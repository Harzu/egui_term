use std::{collections::BTreeMap, sync::mpsc::{self, Receiver, Sender}, thread::JoinHandle};
use egui_term::{TerminalBackend, TerminalView};

#[derive(Debug, Clone)]
pub enum Command {
    BackendEventReceived(u64, egui_term::BackendEvent)
}

pub struct App {
    command_sender: Sender<Command>,
    command_receiver: Receiver<Command>,
    tab_manager: TabManager
}

impl App {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        Self {
            command_sender,
            command_receiver,
            tab_manager: TabManager::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(command) = self.command_receiver.try_recv() {
            match command {
                Command::BackendEventReceived(id, event) => match event {
                    egui_term::BackendEvent::Exit => {
                        self.tab_manager.remove(id);
                    },
                    _ => {}
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tab_ids = self.tab_manager.get_tab_ids();
                for id in tab_ids {
                    if ui.button(format!("{}", id))
                        .clicked()
                    {
                        self.tab_manager.set_active(id.clone());
                    }
                }

                if ui.button("+").clicked() {
                    self.tab_manager.add(self.command_sender.clone(), ctx.clone());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Sense::click(),
            );

            if let Some(tab) = self.tab_manager.get_active() {
                TerminalView::new(
                    &mut tab.backend,
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

struct TabManager {
    active_tab_id: Option<u64>,
    tabs: BTreeMap<u64, Tab>,
}

impl TabManager {
    fn new() -> Self {
        Self {
            active_tab_id: None,
            tabs: BTreeMap::new()
        }
    }

    fn add(&mut self, command_sender: Sender<Command>, ctx: egui::Context) {
        let id = self.tabs.len() as u64;
        let tab = Tab::new(ctx, command_sender, id);
        self.tabs.insert(id, tab);
        self.active_tab_id = Some(id)
    }

    fn remove(&mut self, id: u64) {
        if self.tabs.len() == 0 {
            return;
        }

        self.tabs.remove(&id).unwrap();
        self.active_tab_id = if let Some(next_tab) = self.tabs
            .iter()
            .skip_while(|t| t.0 <= &id)
            .next()
        {
            Some(next_tab.0.clone())
        } else if let Some(last_tab) = self.tabs.last_key_value() {
            Some(last_tab.0.clone())
        } else {
            None
        };
    }

    fn get_active(&mut self) -> Option<&mut Tab> {
        if self.active_tab_id.is_none() {
            return None;
        }

        if let Some(tab) = self.tabs.get_mut(
            &self.active_tab_id.unwrap()
        ) {
            return Some(tab);
        }

        None
    }

    fn get_tab_ids(&self) -> Vec<u64> {
        self.tabs
            .keys()
            .map(|x| *x)
            .collect()
    }

    fn set_active(&mut self, id: u64) {
        if id as usize > self.tabs.len() {
            return;
        }

        self.active_tab_id = Some(id);
    }
}

struct Tab {
    backend: TerminalBackend,
    _event_listener: JoinHandle<()>,
}

impl Tab {
    fn new(ctx: egui::Context, command_sender: Sender<Command>, id: u64) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let backend = TerminalBackend::new(
            id as u64,
            event_tx,
            egui_term::BackendSettings::default(),
        ).unwrap();

        let _event_listener = std::thread::Builder::new()
            .name(format!("backend_event_listener_{}", id))
            .spawn(move || {
                loop {
                    if let Ok(e) = event_rx.recv() {
                        command_sender.send(Command::BackendEventReceived(id as u64, e.clone())).unwrap();
                        match e {
                            egui_term::BackendEvent::Exit => { break; },
                            _ => {},
                        }
                        ctx.clone().request_repaint();
                    }
                }
            }).unwrap();

            Self {
                backend,
                _event_listener,
            }
    }
}