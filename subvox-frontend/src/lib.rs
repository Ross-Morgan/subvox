#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use eframe::egui;

#[derive(Default)]
pub struct App {}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> eframe::Result<Self> {
        let app = Self {};

        Ok(app)
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.heading("Top Panel");
        });

        egui::Panel::bottom("bottom_panel").show(ui, |ui| {
            ui.heading("Bottom Panel");
        });

        egui::Panel::left("left_panel").show(ui, |ui| {
            ui.heading("Left Panel");
        });

        egui::Panel::right("right_panel").show(ui, |ui| {
            ui.heading("Right Panel");
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Subvox");
        });
    }
}
