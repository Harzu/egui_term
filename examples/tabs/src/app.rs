use egui_term::{PtyEvent, TerminalBackend, TerminalView};
use std::{
    collections::BTreeMap,
    sync::mpsc::{self, Receiver, Sender},
};

pub struct App {
    command_sender: Sender<(u64, egui_term::PtyEvent)>,
    command_receiver: Receiver<(u64, egui_term::PtyEvent)>,
    tab_manager: TabManager,
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
        if ctx.input(|i| i.viewport().close_requested()) {
            self.tab_manager.clear();
        }

        if let Ok((tab_id, event)) = self.command_receiver.try_recv() {
            match event {
                egui_term::PtyEvent::Exit => {
                    self.tab_manager.remove(tab_id);
                },
                egui_term::PtyEvent::Title(title) => {
                    self.tab_manager.set_title(tab_id, title);
                },
                _ => {},
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let tab_ids = self.tab_manager.get_tab_ids();
                for id in tab_ids {
                    let tab_title =
                        if let Some(title) = self.tab_manager.get_title(id) {
                            title
                        } else {
                            String::from("unknown")
                        };
                    if ui.button(tab_title).clicked() {
                        self.tab_manager.set_active(id);
                    }
                }

                if ui.button("[+]").clicked() {
                    self.tab_manager
                        .add(self.command_sender.clone(), ctx.clone());
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tab) = self.tab_manager.get_active() {
                let terminal = TerminalView::new(ui, &mut tab.backend)
                    .set_focus(true)
                    .set_size(ui.available_size());

                ui.add(terminal);
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
            tabs: BTreeMap::new(),
        }
    }

    fn add(
        &mut self,
        command_sender: Sender<(u64, PtyEvent)>,
        ctx: egui::Context,
    ) {
        let id = self.tabs.len() as u64;
        let tab = Tab::new(ctx, command_sender, id);
        self.tabs.insert(id, tab);
        self.active_tab_id = Some(id)
    }

    fn remove(&mut self, id: u64) {
        if self.tabs.is_empty() {
            return;
        }

        self.tabs.remove(&id).unwrap();
        self.active_tab_id =
            if let Some(next_tab) = self.tabs.iter().find(|t| t.0 <= &id) {
                Some(*next_tab.0)
            } else {
                self.tabs.last_key_value().map(|last_tab| *last_tab.0)
            };
    }

    fn clear(&mut self) {
        self.tabs.clear();
    }

    fn set_title(&mut self, id: u64, title: String) {
        if let Some(tab) = self.tabs.get_mut(&id) {
            tab.set_title(title);
        }
    }

    fn get_title(&mut self, id: u64) -> Option<String> {
        self.tabs.get(&id).map(|tab| tab.title.clone())
    }

    fn get_active(&mut self) -> Option<&mut Tab> {
        self.active_tab_id?;

        if let Some(tab) = self.tabs.get_mut(&self.active_tab_id.unwrap()) {
            return Some(tab);
        }

        None
    }

    fn get_tab_ids(&self) -> Vec<u64> {
        self.tabs.keys().copied().collect()
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
    title: String,
}

impl Tab {
    fn new(
        ctx: egui::Context,
        command_sender: Sender<(u64, PtyEvent)>,
        id: u64,
    ) -> Self {
        #[cfg(unix)]
        let system_shell =
            std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
        #[cfg(windows)]
        let system_shell = "cmd.exe".to_string();

        let shell_args = if system_shell.contains("bash") {
            vec!["--login".to_string()]
        } else {
            vec![]
        };

        let backend = TerminalBackend::new(
            id,
            ctx,
            command_sender,
            egui_term::BackendSettings {
                shell: system_shell,
                args: shell_args,
                ..Default::default()
            },
        )
        .unwrap();

        Self {
            backend,
            title: format!("tab: {}", id),
        }
    }

    fn set_title(&mut self, title: String) {
        self.title = title;
    }
}
