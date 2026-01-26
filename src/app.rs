use egui::*;

pub struct App {}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        return App {};
    }
}

impl eframe::App for App {
    fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}
}
