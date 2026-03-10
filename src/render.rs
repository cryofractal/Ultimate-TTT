use crate::{board::Board, team::Team};
use egui::{Color32, Rect, Ui, Vec2, vec2};

const DEFAULT_COLOR: Color32 = Color32::GRAY;

impl Board {
    pub fn render(
        &self,
        ui: &mut Ui,
        rect: Rect,
        curr_ind: usize,
        depth: u8,
        team: &Vec<Team>,
        stroke_size: f32,
        corr_box: usize,
    ) {
        let curr_val = self.state_array[curr_ind];
        if curr_val < 254 {
            ui.painter().rect(
                rect,
                0,
                team[curr_val as usize].color,
                (stroke_size, Color32::BLACK),
                egui::StrokeKind::Inside,
            );
        } else {
            if let Some(children) = self.children_base(curr_ind)
                && depth > 0
            {
                for i in 0..self.grid_num() {
                    let new_size =
                        (rect.size() - Vec2::splat(stroke_size * (2.0))) / self.layer as f32;
                    let coord = self.rel_index_to_coord(i);
                    let min = rect.min
                        + vec2(
                            stroke_size + new_size.x * coord.x as f32,
                            stroke_size + new_size.y * coord.y as f32,
                        );
                    let new_rect = Rect::from_min_size(min, new_size);
                    // let new_rect = rect.shrink(amnt)
                    self.render(
                        ui,
                        new_rect,
                        children + i,
                        depth - 1,
                        team,
                        stroke_size / 2.0,
                        corr_box,
                    );
                }
                ui.painter().rect_stroke(
                    rect,
                    0,
                    (
                        stroke_size,
                        if curr_ind == corr_box {
                            Color32::GOLD
                        } else {
                            Color32::BLACK
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
            } else {
                ui.painter().rect(
                    rect,
                    0,
                    DEFAULT_COLOR,
                    (stroke_size, Color32::BLACK),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
}
