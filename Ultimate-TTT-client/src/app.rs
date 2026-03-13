use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use egui::*;

use crate::{PASSWORD, board::Board, game::Game, team::default_teams};

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
const CHECK: u8 = 0_u8.to_le();
const READ: u8 = 1_u8.to_le();
const MOVE: u8 = 2_u8.to_le();
const TERMINATE: u8 = 3_u8.to_le();

pub struct App {
    pub game: Game,
    pub num_moves: Arc<Mutex<usize>>,
    pub curr_ind: usize,
    pub depth: u8,
    pub newboard_rank: u8,
    pub newboard_layer: u8,
    pub newboard_in_a_row: u8,
    pub curr_logfile_path: String,
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub my_team_id: u8,
    pub check_thread_kill: Arc<Mutex<bool>>,
    pub check_thread_check: Arc<Mutex<bool>>,
    curr_ip: String,
    val: usize,
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
            num_moves: Arc::new(Mutex::new(0)),
            curr_ind: 0,
            depth: 3,
            newboard_in_a_row: 3,
            newboard_layer: 3,
            newboard_rank: 3,
            curr_logfile_path: String::new(),
            stream: None,
            my_team_id: 0,
            val: 0,
            check_thread_kill: Arc::new(Mutex::new(false)),
            check_thread_check: Arc::new(Mutex::new(true)),
            curr_ip: String::from(ADDR),
        }
    }
    pub fn input(&mut self, ui: &Ui) {
        if ui.ctx().memory(|x| x.focused().is_none()) {
            ui.input(|input| {
                if input.focused {
                    match self.game.board.layer {
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
            })
        }
    }
    pub fn proc_input_at(&mut self, ind: usize) {
        if self.game.board.state_array[self.curr_ind] == 255
            && let Some(base) = self.game.board.children_base(self.curr_ind)
        {
            if self.game.board.is_leaf(base + ind) {
                if self.game.board.state_array[base + ind] == 255 {
                    self.attempt_move_at(base + ind);
                }
            } else {
                self.curr_ind = base + ind;
            }
        }
    }
    pub fn attempt_move_at(&mut self, ind: usize) {
        self.catch_up();
        if self.game.board.has_ancestor(ind, self.game.correct_box) {
            self.do_move(ind);
            self.game.move_at(ind);
            *self.num_moves.lock().unwrap() += 1;
            self.curr_ind = 0;
        }
    }
    pub fn mouse_input(&mut self, resp: &Response, rect: Rect) {
        if resp.clicked_by(egui::PointerButton::Primary)
            && let Some(hpos) = resp.hover_pos()
            && rect.contains(hpos)
        {
            let square_size = rect.size() / self.game.board.layer as f32;
            let coords = hpos - rect.min;
            let (x, y) = (coords.x / square_size.x, coords.y / square_size.y);
            let (xint, yint) = (x as usize, y as usize);
            if !(xint >= self.game.board.layer as usize || yint >= self.game.board.layer as usize) {
                self.proc_input_at((self.game.board.layer as usize * yint) + xint)
            }
        } else if resp.clicked_by(egui::PointerButton::Secondary)
            && let Some(p) = self.game.board.parent(self.curr_ind)
        {
            self.curr_ind = p;
        }
    }
    pub fn setup_connection_client(&mut self, game_id: usize) {
        let mut stream = TcpStream::connect(&self.curr_ip).unwrap();
        let mut v = Vec::new();
        v.extend_from_slice(&PASSWORD.to_le_bytes());
        v.extend_from_slice(&game_id.to_le_bytes());
        stream.write(v.as_slice()).unwrap();
        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.get_game_data();
        self.restart_check_thread();
        println!("Client connected!");
    }
    pub fn restart_check_thread(&mut self) {
        *self.check_thread_kill.lock().unwrap() = true;
        self.check_thread_kill = Arc::new(Mutex::new(false));
        let check_check = self.check_thread_check.clone();
        let second_stream = self.stream.as_ref().unwrap().clone();
        let check_kill = self.check_thread_kill.clone();
        let num_moves2 = self.num_moves.clone();
        thread::spawn(move || {
            loop {
                if *check_kill.lock().unwrap() == true {
                    break;
                } else {
                    let val = check(
                        &mut second_stream.lock().unwrap(),
                        *num_moves2.lock().unwrap(),
                    );
                    *check_check.lock().unwrap() = val;
                }
                sleep(Duration::from_millis(100));
            }
        });
    }
    pub fn get_game_data(&mut self) {
        if let Some(ref str) = self.stream {
            let mut stream = str.lock().unwrap();
            let mut header = [0; 4];
            while stream.read(&mut header).unwrap() == 0 {}
            let board = Board::new(
                u8::from_le(header[1]),
                u8::from_le(header[0]),
                u8::from_le(header[2]),
            );
            self.game = Game::new(board, default_teams(), 0, vec![], 0);
            let mut length_buffer = [0; 8];
            let mut move_buffer = [0; 8];
            while stream.read(&mut length_buffer).unwrap() == 0 {}
            let len = usize::from_le_bytes(length_buffer);
            *self.num_moves.lock().unwrap() = len;
            for _ in 0..len {
                while stream.read(&mut move_buffer).unwrap() == 0 {}
                let mov = usize::from_le_bytes(move_buffer.clone());
                self.game.move_at(mov);
            }
            self.game.curr_team = u8::from_le(header[3]);
        }
    }
    pub fn check(&mut self) -> bool {
        if let Some(ref str) = self.stream {
            check(&mut str.lock().unwrap(), *self.num_moves.lock().unwrap())
        } else {
            todo!()
        }
    }
    pub fn read_moves(&mut self) {
        if let Some(ref mut str) = self.stream {
            let mut stream = str.lock().unwrap();
            let mut team_id_buf = [0; 1];
            let mut buf = [0; 8];
            let mut v = Vec::new();
            v.push(READ);
            v.extend_from_slice(&self.game.prev_moves.len().to_le_bytes());
            stream.write(v.as_slice()).unwrap();
            while stream.read(&mut team_id_buf).unwrap() == 0 {}
            while stream.read(&mut buf).unwrap() == 0 {}
            let len = usize::from_le_bytes(buf.clone());
            *self.num_moves.lock().unwrap() += len;
            for _ in 0..len {
                while stream.read(&mut buf).unwrap() == 0 {}
                let mov = usize::from_le_bytes(buf.clone());
                self.game.move_at(mov);
            }
            self.game.curr_team = u8::from_le(team_id_buf[0]);
        } else {
            todo!()
        }
    }
    pub fn do_move(&mut self, mov: usize) {
        if let Some(ref str) = self.stream {
            let mut stream = str.lock().unwrap();
            let mut v = Vec::new();
            v.push(MOVE);
            v.extend_from_slice(&mov.to_le_bytes());
            stream.write(v.as_slice()).unwrap();
        }
    }
    pub fn terminate(&mut self) {
        if let Some(ref str) = self.stream {
            let mut stream = str.lock().unwrap();
            let mut v = Vec::new();
            v.push(TERMINATE);
            v.extend_from_slice(&0_usize.to_le_bytes());
            stream.write(v.as_slice()).unwrap();
        }
    }
    pub fn catch_up(&mut self) {
        let check_mutex = self.check_thread_check.lock().unwrap();
        let check = *check_mutex;
        drop(check_mutex);
        if !check {
            self.read_moves();
            *self.check_thread_check.lock().unwrap() = true;
        }
    }
}

pub fn check(stream: &mut TcpStream, len: usize) -> bool {
    let mut buf = [0; 1];
    let mut v = Vec::new();
    v.push(CHECK);
    v.extend_from_slice(&len.to_le_bytes());
    stream.write(v.as_slice()).unwrap();
    while stream.read(&mut buf).unwrap() == 0 {}
    let val = u8::from_le(buf[0]);
    if val == 0 { false } else { true }
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
            self.catch_up();
            ui.label("Rendering Depth");
            ui.add(Slider::new(&mut self.depth, 1..=self.game.board.rank));
            ui.label("Log File Path");
            ui.text_edit_singleline(&mut self.curr_logfile_path);
            // if ui.button("Read from file").clicked()
            //     && let Some(new) =
            //         App::from_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")))
            // {
            //     *self = new;
            // }
            // if ui.button("Write to file").clicked() {
            //     self.write_to_file(&PathBuf::from(&(self.curr_logfile_path.clone() + ".txt")));
            // }
            ui.add(TextEdit::singleline(&mut self.curr_ip));
            if ui.button("Connect to server").clicked() {
                self.setup_connection_client(0);
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
            let resp = ui.interact(display_rect, Id::new(10), Sense::all());
            self.input(ui);
            self.mouse_input(&resp, display_rect);
            if ui.button("CHECK").clicked() {
                dbg!(self.check());
            }
            if ui.button("READ").clicked() {
                self.read_moves();
            }
            if ui.button("MOVE").clicked() {
                self.do_move(self.val);
            }
            if ui.button("TERMINATE").clicked() {
                self.terminate();
            }
            ui.add(Slider::new(&mut self.val, 0..=100));
            //self.check_connection();
        });
        ctx.request_repaint();
    }
}
