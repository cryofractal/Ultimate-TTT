use std::collections::HashMap;

use crate::{
    cell::{Cell, Coord},
    team::Team,
};
use egui::{Color32, Pos2, Rect, Ui, pos2, vec2};

impl Coord {
    pub fn to_pos2(&self, scale: f32, base: Pos2) -> Pos2 {
        if self.coord.len() != 2 {
            panic!("This is not a 2d position!")
        } else {
            base + scale * vec2(self.coord[0] as f32, -(self.coord[1] as f32))
        }
    }
}

impl Cell {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: Rect,
        scale: f32,
        base: Pos2,
        teams: &HashMap<u8, Team>,
        divide: u8,
    ) {
        match self.state {
            crate::cell::CellState::Owned(t) => {
                ui.painter().rect(
                    rect,
                    0.0,
                    teams.get(&t).unwrap().color,
                    (scale * 0.1, Color32::BLACK),
                    egui::StrokeKind::Middle,
                );
            }
            crate::cell::CellState::Empty => {}
            crate::cell::CellState::Contested => {
                for (pos, cell) in self.children.iter() {
                    let bottom = pos.to_pos2(scale, base);
                    let top = Coord {
                        coord: vec![pos.coord[0] + 1, pos.coord[1] + 1],
                    }
                    .to_pos2(scale, base);
                    cell.render(
                        ui,
                        Rect::from_points(&[top, bottom]),
                        scale / (divide as f32),
                        bottom,
                        teams,
                        divide,
                    );
                }
            }
        }
    }
}
