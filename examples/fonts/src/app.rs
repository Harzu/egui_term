use egui::{FontId, Vec2};
use egui_term::{
    FontSettings, PtyEvent, TerminalBackend, TerminalFont, TerminalView,
};
use std::sync::mpsc::Receiver;

const TERM_FONT_JET_BRAINS_BYTES: &[u8] = include_bytes!(
    "../assets/fonts/JetBrains/JetBrainsMonoNerdFontMono-Bold.ttf"
);

const TERM_FONT_3270_BYTES: &[u8] =
    include_bytes!("../assets/fonts/3270/3270NerdFont-Regular.ttf");

fn setup_font(ctx: &egui::Context, name: &str) {
    let bytes = if name == "3270" {
        TERM_FONT_3270_BYTES
    } else {
        TERM_FONT_JET_BRAINS_BYTES
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert(name.to_owned(), egui::FontData::from_static(bytes));

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, name.to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(name.to_owned());

    ctx.set_fonts(fonts);
}

pub struct App {
    terminal_backend: TerminalBackend,
    font_size: f32,
    pty_proxy_receiver: Receiver<(u64, egui_term::PtyEvent)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_font(&cc.egui_ctx, "jet_brains_mono");
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
            font_size: 14.0,
            pty_proxy_receiver,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok((_, PtyEvent::Exit)) = self.pty_proxy_receiver.try_recv() {
            // if let PtyEvent::Exit = event {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
            // }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("jet brains").clicked() {
                    setup_font(ctx, "jet_brains_mono");
                }

                if ui.button("3270").clicked() {
                    setup_font(ctx, "3270");
                }
            });

            ui.horizontal(|ui| {
                if ui.button("+ size").clicked() {
                    self.font_size += 1.0;
                }

                if ui.button("- size").clicked() {
                    self.font_size -= 1.0;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let terminal = TerminalView::new(ui, &mut self.terminal_backend)
                .set_focus(true)
                .set_font(TerminalFont::new(FontSettings {
                    font_type: FontId::proportional(self.font_size),
                }))
                .set_size(Vec2::new(
                    ui.available_width(),
                    ui.available_height(),
                ));

            ui.add(terminal);
        });
    }
}
