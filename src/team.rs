use egui::Color32;

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub id: u8,
    pub color: Color32,
}

pub fn default_teams() -> Vec<Team> {
    vec![
        Team {
            name: "X".to_string(),
            id: 0,
            color: Color32::RED,
        },
        Team {
            name: "O".to_string(),
            id: 0,
            color: Color32::BLUE,
        },
    ]
}
