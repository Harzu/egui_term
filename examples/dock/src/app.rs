use eframe::epaint::FontId;
use egui::{Context, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use egui_term::{
    BackendSettings, FontSettings, PtyEvent, TerminalBackend, TerminalFont,
    TerminalView,
};
use log::error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

pub static GLOBAL_COUNTER: Counter = Counter::new();

pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub const fn new() -> Counter {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn next(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
}

pub struct Tab {
    backend: TerminalBackend,
    id: u64,
}

impl Tab {
    pub fn term(ctx: Context, command_sender: Sender<(u64, PtyEvent)>) -> Self {
        let id = GLOBAL_COUNTER.next();
        let backend = TerminalBackend::new(
            id,
            ctx,
            command_sender,
            BackendSettings::default(),
        )
        .unwrap();

        Self { id, backend }
    }
}

struct TabViewer<'a> {
    command_sender: &'a Sender<(u64, PtyEvent)>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("tab {}", tab.id).into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        let terminal = TerminalView::new(ui, &mut tab.backend)
            .set_focus(true)
            .set_font(TerminalFont::new(FontSettings {
                font_type: FontId::monospace(20f32),
            }))
            .set_size(ui.available_size());
        ui.add(terminal);
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        match self.command_sender.send((tab.id, PtyEvent::Exit)) {
            Err(err) => {
                error!("close tab {} failed: {err}", tab.id);
                false
            },
            Ok(_) => true,
        }
    }
}

pub struct App {
    command_sender: Sender<(u64, PtyEvent)>,
    dock_state: DockState<Tab>,
}

impl App {
    pub fn new(ctx: &Context) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        thread::spawn(move || {
            while command_receiver.recv().is_ok() {
                // do something
            }
        });

        let mut dock_state = DockState::new(vec![Tab::term(
            ctx.clone(),
            command_sender.clone(),
        )]);

        dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.5,
            vec![Tab::term(ctx.clone(), command_sender.clone())],
        );

        Self {
            command_sender,
            dock_state,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state)
            .show_add_buttons(false)
            .show_window_collapse_buttons(false)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(
                ctx,
                &mut TabViewer {
                    command_sender: &self.command_sender,
                },
            );
    }
}
