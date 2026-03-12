use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use egui::{CentralPanel, Id, Key, Rect, Response, Sense, Slider, Ui, Vec2};

use crate::{
    board::Board,
    //render::render_buttons,
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
    pub board: Board,
    pub teams: Vec<Team>,
    pub curr_ind: usize,
    pub curr_team: u8,
    pub prev_moves: Vec<usize>,
    pub depth: u8,
    pub newboard_rank: u8,
    pub newboard_layer: u8,
    pub newboard_in_a_row: u8,
    pub correct_box: usize,
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
            board: Board::new(2, 3, 3),
            teams: default_teams(),
            curr_team: 0,
            curr_ind: 0,
            prev_moves: vec![],
            depth: 3,
            newboard_in_a_row: 3,
            newboard_layer: 3,
            newboard_rank: 3,
            correct_box: 0,
            curr_logfile_path: String::new(),
            stream: None,
            my_team_id: 0,
        }
    }
    //returns if a move happened
    pub fn proc_input_at(&mut self, ind: usize) -> bool {
        if self.board.state_array[self.curr_ind] == 255
            && let Some(base) = self.board.children_base(self.curr_ind)
        {
            if self.board.is_leaf(base + ind) {
                if self.board.state_array[base + ind] == 255
                    && self.board.has_ancestor(base + ind, self.correct_box)
                {
                    self.move_at(base + ind, true);
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
    pub fn move_at(&mut self, ind: usize, direct: bool) {
        self.board.move_at_index(ind, self.curr_team);
        self.correct_box = self.board.get_next_correct_move_box(ind).unwrap();
        self.prev_moves.push(ind);
        self.curr_team = (self.curr_team + 1) % self.teams.len() as u8;
        self.curr_ind = 0;
        if direct {
            self.write_to_connection(ind);
        }
    }
    pub fn input(&mut self, ui: &Ui) -> bool {
        if ui.ctx().memory(|x| x.focused().is_none()) {
            ui.input(|input| {
                if input.focused {
                    match self.board.layer {
                        3 => {
                            for (key, ind) in KEYBINDS_3 {
                                if input.key_pressed(key) {
                                    return self.proc_input_at(ind);
                                }
                            }
                        }
                        2 => {
                            for (key, ind) in KEYBINDS_2 {
                                if input.key_pressed(key) {
                                    return self.proc_input_at(ind);
                                }
                            }
                        }
                        _ => {}
                    }
                    if input.key_pressed(BACK_KEY)
                        && let Some(p) = self.board.parent(self.curr_ind)
                    {
                        self.curr_ind = p;
                    } else if input.key_pressed(UNDO_KEY)
                        && let Some(undo) = self.prev_moves.pop()
                    {
                        self.board.undo_at(undo);
                        self.curr_team = if self.curr_team == 0 {
                            (self.teams.len() - 1) as u8
                        } else {
                            self.curr_team - 1
                        };
                        self.correct_box = if let Some(prev) = self.prev_moves.last() {
                            self.board.get_next_correct_move_box(*prev).unwrap()
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

    pub fn mouse_input(&mut self, resp: &Response, rect: Rect) -> bool {
        if resp.clicked_by(egui::PointerButton::Primary)
            && let Some(hpos) = resp.hover_pos()
            && rect.contains(hpos)
        {
            let square_size = rect.size() / self.board.layer as f32;
            let coords = hpos - rect.min;
            let (x, y) = (coords.x / square_size.x, coords.y / square_size.y);
            let (xint, yint) = (x as usize, y as usize);
            if !(xint >= self.board.layer as usize || yint >= self.board.layer as usize) {
                self.proc_input_at((self.board.layer as usize * yint) + xint)
            } else {
                false
            }
        } else if resp.clicked_by(egui::PointerButton::Secondary)
            && let Some(p) = self.board.parent(self.curr_ind)
        {
            self.curr_ind = p;
            false
        } else {
            false
        }
    }
    pub fn check_connection(&mut self) -> bool {
        let mut buf = [0; 8];
        if self.curr_team != self.my_team_id
            && let Some(ref mut stream) = self.stream
            && let Ok(x) = stream.read(&mut buf)
            && x > 0
        {
            let ind = usize::from_le_bytes(buf);
            dbg!(ind);
            self.move_at(ind, false);
            true
        } else {
            false
        }
    }
    pub fn write_to_connection(&mut self, ind: usize) -> bool {
        if let Some(ref mut stream) = self.stream
            && let Ok(x) = stream.write(&ind.to_le_bytes())
            && x > 0
        {
            true
        } else {
            false
        }
    }

    pub fn setup_connection_host(&mut self) {
        let listener = TcpListener::bind(ADDR).unwrap();
        let (stream, _addr) = listener.accept().unwrap();
        self.stream = Some(stream);
        self.my_team_id = 0;
        println!("Host connected!");
    }
    pub fn setup_connection_client(&mut self) {
        let stream = TcpStream::connect(ADDR).unwrap();
        self.stream = Some(stream);
        self.my_team_id = 1;
        println!("Client connected!");
    }
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
                self.board = Board::new(
                    self.newboard_rank,
                    self.newboard_layer,
                    self.newboard_in_a_row,
                );
                self.curr_team = 0;
                self.correct_box = 0;
                self.curr_ind = 0;
                self.depth = self.newboard_rank.min(5);
                self.prev_moves = vec![];
            }
            ui.label("Rendering Depth");
            ui.add(Slider::new(&mut self.depth, 1..=self.board.rank));
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
            if ui.button("Host").clicked() {
                self.setup_connection_host();
            }
            if ui.button("Client").clicked() {
                self.setup_connection_client();
            }

            let rect_size = Vec2::splat(screen_size.size().y);
            let display_rect = Rect::from_center_size(screen_size.center(), rect_size);
            self.board.render(
                ui,
                display_rect,
                self.curr_ind,
                self.depth,
                &self.teams,
                10.0,
                self.correct_box,
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
