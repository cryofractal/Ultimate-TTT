use egui::*;
#[derive(Clone, Debug)]
pub struct Team {
    name: String,
    pub id: u8,
    color: egui::Color32,
}
