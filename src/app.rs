use std::path::PathBuf;

use egui::{CentralPanel, Id, Key, Rect, Response, Sense, Slider, Ui, Vec2};

use crate::{
    board::Board,
    //render::render_buttons,
    team::{Team, default_teams},
};

const KEYBINDS: [(Key, usize); 9] = [
    (Key::R, 0),
    (Key::T, 1),
    (Key::Y, 2),
    (Key::F, 3),
    (Key::G, 4),
    (Key::H, 5),
    (Key::V, 6),
    (Key::B, 7),
    (Key::N, 8),
];

const BACK_KEY: Key = Key::Backspace;
const UNDO_KEY: Key = Key::Z;
const CORRECT_BOX_KEY: Key = Key::Enter;

pub struct Game {
    pub board: Board,
    pub teams: Vec<Team>,
    pub curr_ind: usize,
    pub curr_team: u8,
    pub prev_moves: Vec<usize>,
    pub correct_box: usize,
}

impl Game {
    pub fn new(rank: u8, layers: u8, in_a_row: u8) -> Self {
        Game {
            board: Board::new(rank, layers, in_a_row),
            teams: default_teams(),
            curr_team: 0,
            curr_ind: 0,
            prev_moves: vec![],
            correct_box: 0,
        }
    }
    pub fn proc_input_at(&mut self, ind: usize) {
        if self.board.state_array[self.curr_ind] == 255
            && let Some(base) = self.board.children_base(self.curr_ind)
        {
            if self.board.is_leaf(base + ind) {
                if self.board.state_array[base + ind] == 255
                    && self.board.has_ancestor(base + ind, self.correct_box)
                {
                    self.board.move_at_index(base + ind, self.curr_team);
                    self.correct_box = self.board.get_next_correct_move_box(base + ind).unwrap();
                    self.prev_moves.push(base + ind);
                    self.curr_team = (self.curr_team + 1) % self.teams.len() as u8;
                    self.curr_ind = 0;
                }
            } else {
                self.curr_ind = base + ind;
            }
        }
    }

    pub fn input(&mut self, ui: &Ui) {
        if ui.ctx().memory(|x| x.focused().is_none()) {
            ui.input(|input| {
                if input.focused {
                    for (key, ind) in KEYBINDS {
                        if input.key_pressed(key) {
                            self.proc_input_at(ind);
                        }
                    }
                    if input.key_pressed(BACK_KEY)
                        && let Some(p) = self.board.parent(self.curr_ind)
                    {
                        self.curr_ind = p;
                    } else if input.key_pressed(UNDO_KEY)
                        && let Some(undo) = self.prev_moves.pop()
                    {
                        self.board.undo_at(undo);
                        self.correct_box = if let Some(prev) = self.prev_moves.last() {
                            self.board.get_next_correct_move_box(*prev).unwrap()
                        } else {
                            0
                        }
                    } else if input.key_pressed(CORRECT_BOX_KEY) {
                        self.curr_ind = self.correct_box;
                    }
                }
            })
        }
    }

    pub fn mouse_input(&mut self, resp: &Response, rect: Rect) {
        if resp.clicked_by(egui::PointerButton::Primary)
            && let Some(hpos) = resp.hover_pos()
            && rect.contains(hpos)
        {
            let square_size = rect.size() / self.board.layer as f32;
            let coords = hpos - rect.min;
            let (x, y) = (coords.x / square_size.x, coords.y / square_size.y);
            let (xint, yint) = (x as usize, y as usize);
            if !(xint >= self.board.layer as usize || yint >= self.board.layer as usize) {
                self.proc_input_at((self.board.layer as usize * yint) + xint);
            }
        } else if resp.clicked_by(egui::PointerButton::Secondary)
            && let Some(p) = self.board.parent(self.curr_ind)
        {
            self.curr_ind = p;
        }
    }
}

pub struct App {
    pub game: Game,
    pub depth: u8,
    pub newboard_rank: u8,
    pub newboard_layer: u8,
    pub newboard_in_a_row: u8,
    pub curr_logfile_path: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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
            game: Game::new(2, 3, 3),
            depth: 3,
            newboard_in_a_row: 3,
            newboard_layer: 3,
            newboard_rank: 3,
            curr_logfile_path: String::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let screen_size = ui.available_rect_before_wrap();
            let rect_size = Vec2::splat(screen_size.size().y);
            let display_rect = Rect::from_center_size(screen_size.center(), rect_size);
            let resp = ui.interact(display_rect, Id::new(10), Sense::all());
            ui.label("In a Row");
            ui.add(Slider::new(&mut self.newboard_in_a_row, 2..=15));
            ui.label("Layer");
            ui.add(Slider::new(&mut self.newboard_layer, 2..=15));
            ui.label("Rank");
            ui.add(Slider::new(&mut self.newboard_rank, 1..=12));
            if ui.button("Generate New Board").clicked() {
                self.game.board = Board::new(
                    self.newboard_rank,
                    self.newboard_layer,
                    self.newboard_in_a_row,
                );
                self.game.curr_team = 0;
                self.game.correct_box = 0;
                self.game.curr_ind = 0;
                self.depth = self.newboard_rank.min(5);
                self.game.prev_moves = vec![];
            }
            ui.label("Rendering Depth");
            ui.add(Slider::new(&mut self.depth, 1..=self.game.board.rank));
            ui.label("Log File Path");
            ui.text_edit_singleline(&mut self.curr_logfile_path);
            if ui.button("Read from file").clicked()
                && let Some(new) =
                    Game::from_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")))
            {
                self.game = new;
            }
            if ui.button("Write to file").clicked() {
                self.game
                    .write_to_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")));
            }

            self.game.board.render(
                ui,
                display_rect,
                self.game.curr_ind,
                self.depth,
                &self.game.teams,
                10.0,
                self.game.correct_box,
            );
            self.game.input(ui);
            self.game.mouse_input(&resp, display_rect);
        });
    }
}
