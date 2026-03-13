use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use egui::*;

use crate::{
    board::Board,
    game::Game,
    team::{Team, default_teams},
};

const KEYBINDS_3: [(Key, usize); 9] = [
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

const KEYBINDS_2: [(Key, usize); 4] = [(Key::R, 0), (Key::T, 1), (Key::F, 2), (Key::G, 3)];

const BACK_KEY: Key = Key::Backspace;
const UNDO_KEY: Key = Key::Z;

const ADDR: &str = "127.0.0.1:52525";

pub struct App {
    pub game: Game,
    pub curr_ind: usize,
    pub depth: u8,
    pub newboard_rank: u8,
    pub newboard_layer: u8,
    pub newboard_in_a_row: u8,
    pub curr_logfile_path: String,
    pub stream: Option<TcpStream>,
    pub my_team_id: u8,
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
            game: Game::default_game(),
            curr_ind: 0,
            depth: 3,
            newboard_in_a_row: 3,
            newboard_layer: 3,
            newboard_rank: 3,
            curr_logfile_path: String::new(),
            stream: None,
            my_team_id: 0,
        }
    }
    pub fn input(&mut self, ui: &Ui) -> bool {
        if ui.ctx().memory(|x| x.focused().is_none()) {
            ui.input(|input| {
                if input.focused {
                    match self.game.board.layer {
                        3 => {
                            for (key, ind) in KEYBINDS_3 {
                                if input.key_pressed(key) {
                                    return self.game.proc_input_at(ind);
                                }
                            }
                        }
                        2 => {
                            for (key, ind) in KEYBINDS_2 {
                                if input.key_pressed(key) {
                                    return self.game.proc_input_at(ind);
                                }
                            }
                        }
                        _ => {}
                    }
                    if input.key_pressed(BACK_KEY)
                        && let Some(p) = self.game.board.parent(self.curr_ind)
                    {
                        self.curr_ind = p;
                    } else if input.key_pressed(UNDO_KEY)
                        && let Some(undo) = self.game.prev_moves.pop()
                    {
                        self.game.board.undo_at(undo);
                        self.game.curr_team = if self.game.curr_team == 0 {
                            (self.game.teams.len() - 1) as u8
                        } else {
                            self.game.curr_team - 1
                        };
                        self.game.correct_box = if let Some(prev) = self.game.prev_moves.last() {
                            self.game.board.get_next_correct_move_box(*prev).unwrap()
                        } else {
                            0
                        }
                    }
                }
                return false;
            })
        } else {
            false
        }
    }
    pub fn proc_input_at(&mut self, ind: usize) -> bool {
        if self.game.board.state_array[self.curr_ind] == 255
            && let Some(base) = self.game.board.children_base(self.curr_ind)
        {
            if self.game.board.is_leaf(base + ind) {
                if self.game.board.state_array[base + ind] == 255
                    && self.game.board.has_ancestor(base + ind, self.game.correct_box)
                {
                    self.game.move_at(base + ind);
                    true
                } else {
                    false
                }
            } else {
                self.curr_ind = base + ind;
                false
            }
        } else {
            false
        }
    }
    pub fn mouse_input(&mut self, resp: &Response, rect: Rect) -> bool {
        if resp.clicked_by(egui::PointerButton::Primary)
            && let Some(hpos) = resp.hover_pos()
            && rect.contains(hpos)
        {
            let square_size = rect.size() / self.game.board.layer as f32;
            let coords = hpos - rect.min;
            let (x, y) = (coords.x / square_size.x, coords.y / square_size.y);
            let (xint, yint) = (x as usize, y as usize);
            if !(xint >= self.game.board.layer as usize || yint >= self.game.board.layer as usize) {
                self.game
                    .proc_input_at((self.game.board.layer as usize * yint) + xint)
            } else {
                false
            }
        } else if resp.clicked_by(egui::PointerButton::Secondary)
            && let Some(p) = self.game.board.parent(self.curr_ind)
        {
            self.curr_ind = p;
            false
        } else {
            false
        }
    }
    pub fn check_connection(&mut self) -> bool {
        let mut buf = [0; 8];
        if self.game.curr_team != self.my_team_id
            && let Some(ref mut stream) = self.stream
            && let Ok(x) = stream.read(&mut buf)
            && x > 0
        {
            let ind = usize::from_le_bytes(buf);
            dbg!(ind);
            self.game.move_at(ind);
            true
        } else {
            false
        }
    }
    pub fn write_to_connection(&mut self, ind: usize) -> bool {
        let bytes = ind.to_le_bytes();
        let mut buf = [0; 9];
        buf[0] = 2;
        for i in 0..8 {
            buf[i + 1] = bytes[i]
        }
        if let Some(ref mut stream) = self.stream
            && let Ok(x) = stream.write(&buf)
            && x > 0
        {
            true
        } else {
            false
        }
    }
    pub fn setup_connection_client(&mut self, game_id: usize) {
        let stream = TcpStream::connect(ADDR).unwrap();
        self.stream = Some(stream);
        self.my_team_id = 1;
        println!("Client connected!");
    }
    pub fn get_game_data(&mut self) {
        if let Some(ref mut stream) = self.stream {
            let mut header = [0; 4];
            while stream.read(&mut header).unwrap() == 0 {}
            let mut board = Board::new(
                u8::from_le(header[1]),
                u8::from_le(header[0]),
                u8::from_le(header[2]),
            );
            self.game.board = board;
            let mut length_buffer = [0; 8];
            let mut move_buffer = [0; 8];
            while stream.read(&mut length_buffer).unwrap() == 0 {}
            let len = usize::from_le_bytes(length_buffer);
            for i in 0..len {
                while stream.read(&mut move_buffer).unwrap() == 0 {}
                let mov = usize::from_le_bytes(move_buffer.clone());
                self.game.move_at(mov);
            }
            self.game.curr_team = u8::from_le(header[3]);
        }
    }
    pub fn check(&mut self) -> bool {}
    pub fn read_moves(&mut self) {}
    pub fn do_move(&mut self, mov: usize) {}
    pub fn terminate(&mut self) {}
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let screen_size = ui.available_rect_before_wrap();
            ui.label("In a Row");
            ui.add(Slider::new(&mut self.newboard_in_a_row, 2..=15));
            ui.label("Layer");
            ui.add(Slider::new(&mut self.newboard_layer, 2..=15));
            ui.label("Rank");
            ui.add(Slider::new(&mut self.newboard_rank, 2..=12));
            if ui.button("Generate New Board").clicked() {
                self.game.board = Board::new(
                    self.newboard_rank,
                    self.newboard_layer,
                    self.newboard_in_a_row,
                );
                self.game.curr_team = 0;
                self.game.correct_box = 0;
                self.game.prev_moves = vec![];
                self.curr_ind = 0;
                self.depth = self.newboard_rank.min(5);
            }
            ui.label("Rendering Depth");
            ui.add(Slider::new(&mut self.depth, 1..=self.game.board.rank));
            ui.label("Log File Path");
            ui.text_edit_singleline(&mut self.curr_logfile_path);
            if ui.button("Read from file").clicked()
                && let Some(new) =
                    App::from_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")))
            {
                *self = new;
            }
            if ui.button("Write to file").clicked() {
                self.write_to_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")));
            }
            if ui.button("Connect to server").clicked() {
                self.setup_connection_client();
            }

            let rect_size = Vec2::splat(screen_size.size().y);
            let display_rect = Rect::from_center_size(screen_size.center(), rect_size);
            self.game.board.render(
                ui,
                display_rect,
                self.curr_ind,
                self.depth,
                &self.game.teams,
                10.0,
                self.game.correct_box,
            );
            let resp = ui.interact(ui.available_rect_before_wrap(), Id::new(10), Sense::all());
            let mut _early_ret = false;
            if self.input(ui) {
                _early_ret = true;
            }
            if self.mouse_input(&resp, display_rect) {
                _early_ret = true;
            }
            self.check_connection();
        });
    }
}
