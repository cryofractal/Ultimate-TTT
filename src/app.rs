use std::collections::HashMap;

use egui::{CentralPanel, Color32, Pos2, Rect, Ui, pos2};
use itertools::Itertools;

use crate::{
    cell::{Cell, Coord},
    coord,
    render::render_buttons,
    team::Team,
    test::{self, generate_rank_n},
};

pub struct App {
    cell: Cell,
    scale: f32,
    divisions: u8,
    base: Pos2,
    teams: HashMap<u8, Team>,
    temp_path: Vec<Coord>,
    curr_team: u8,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let mut cell = generate_rank_n(2);
        // let moves = vec![
        //     vec![coord![0, 1], coord![1, 0]],
        //     vec![coord![0, 1], coord![1, 2]],
        //     vec![coord![0, 1], coord![1, 1]],
        //     vec![coord![0, 0], coord![2, 2]],
        // ];
        // for m in moves {
        //     cell.update(&m, 0);
        // }
        // dbg!(
        //     &cell
        //         .children
        //         .get(&coord![0, 0])
        //         .unwrap()
        //         .children
        //         .get(&coord![2, 2])
        // );
        // dbg!(&cell.state);
        App {
            cell: generate_rank_n(2),
            scale: 300.0,
            divisions: 3,
            base: { pos2(0.0, cc.egui_ctx.screen_rect().max.y) },
            teams: HashMap::from([
                (
                    0,
                    Team {
                        name: String::from("X"),
                        id: 0,
                        color: Color32::RED,
                    },
                ),
                (
                    1,
                    Team {
                        name: String::from("O"),
                        id: 1,
                        color: Color32::BLUE,
                    },
                ),
            ]),
            temp_path: Vec::new(),
            curr_team: 0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.cell.render(
                ui,
                Rect {
                    min: pos2(100.0, 100.0),
                    max: pos2(1000.0, 1000.0),
                },
                self.scale,
                pos2(100.0, 1000.0),
                &self.teams,
                self.divisions,
            );
            self.cell.render_border(
                ui,
                Rect {
                    min: pos2(100.0, 100.0),
                    max: pos2(1000.0, 1000.0),
                },
                self.scale,
                pos2(100.0, 1000.0),
                self.divisions,
            );
            let curr_cell = self.cell.get(&self.temp_path);
            render_buttons(
                ui,
                curr_cell,
                &mut self.temp_path,
                pos2(1200.0, 500.0),
                100.0,
            );
            if curr_cell.children.is_empty() {
                self.cell.update(&self.temp_path, self.curr_team);
                self.curr_team = 1 - self.curr_team;
                self.temp_path = Vec::new();
            }
            if ui.button("Reset").clicked() {
                self.temp_path = Vec::new()
            }
            if ui.button("Undo").clicked() {
                self.cell.undo();
                self.curr_team = 1 - self.curr_team;
            }
        });
    }
}
