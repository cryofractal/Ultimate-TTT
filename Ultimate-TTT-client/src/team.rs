use egui::Color32;

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub id: u8,
    pub color: Color32,
}

fn default_teams_full() -> Vec<Team> {
    vec![
        Team {
            name: "X".to_string(),
            id: 0,
            color: Color32::RED,
        },
        Team {
            name: "O".to_string(),
            id: 1,
            color: Color32::BLUE,
        },
        Team {
            name: "T".to_string(),
            id: 2,
            color: Color32::GREEN,
        },
        Team {
            name: "U".to_string(),
            id: 3,
            color: Color32::YELLOW,
        },
    ]
}

pub fn default_teams(num: u8) -> Vec<Team> {
    Vec::from(&default_teams_full()[0..(num as usize)])
}
