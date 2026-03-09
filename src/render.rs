use std::collections::HashMap;

use crate::{board::Board, team::Team};
use egui::{Button, Color32, Pos2, Rect, Ui, pos2, vec2};

const DEFAULT_COLOR: Color32 = Color32::GRAY;

impl Board {
    pub fn render(&self, ui: &mut Ui, rect: Rect, curr_ind: usize, depth: u8, team: &Vec<Team>) {
        let curr_val = self.state_array[curr_ind];
        if curr_val > 1 {
            ui.painter().rect(
                rect,
                0,
                team[2].color,
                (10.0, Color32::BLACK),
                egui::StrokeKind::Inside,
            );
        } else {
            if let Some(children) = self.children_base(curr_ind)
                && depth > 0
            {
                for i in 0..self.grid_num() {
                    let new_size = rect.size() / self.layer as f32;
                    let coord = self.rel_index_to_coord(i);
                    let min =
                        rect.min + vec2(coord.x as f32 * new_size.x, coord.y as f32 * new_size.y);
                    let new_rect = Rect::from_min_size(min, new_size);
                    self.render(ui, new_rect, children + i, depth - 1, team);
                }
            } else {
                ui.painter().rect(
                    rect,
                    0,
                    DEFAULT_COLOR,
                    (10.0, Color32::BLACK),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
}
