use egui::*;
#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub id: u8,
    pub color: egui::Color32,
}
