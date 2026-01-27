use std::collections::HashMap;

use egui::{CentralPanel, Color32, Pos2, Rect, pos2};

use crate::{cell::Cell, team::Team};

pub struct App {
    cell: Cell,
    scale: f32,
    divisions: u8,
    base: Pos2,
    teams: HashMap<u8, Team>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        App {
            cell: todo!(),
            scale: 50.0,
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
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            self.cell.render(
                ui,
                Rect {
                    min: pos2(200.0, 200.0),
                    max: pos2(1000.0, 1000.0),
                },
                self.scale,
                pos2(200.0, 1000.0),
                &self.teams,
                self.divisions,
            )
        });
    }
}
