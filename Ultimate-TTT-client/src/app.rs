use std::{
    io::{Read, Write},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread::{self, sleep},
    time::Duration,
};

use egui::*;

use crate::{PASSWORD, board::Board, game::Game, team::default_teams};
const ORDER: Ordering = Ordering::Relaxed;

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
//const UNDO_KEY: Key = Key::Z;

const ADDR: &str = "68.183.121.208:52525";
const CHECK: u8 = 0_u8.to_le();
const READ: u8 = 1_u8.to_le();
const MOVE: u8 = 2_u8.to_le();
const TERMINATE: u8 = 3_u8.to_le();
const NEWGAME: u8 = 0_u8.to_le();
const JOINGAME: u8 = 1_u8.to_le();
const NO_SUCH_GAME: u8 = 0_u8.to_le();
const GAME_EXISTS: u8 = 1_u8.to_le();

pub struct App {
    pub game: Game,
    pub num_moves: Arc<AtomicUsize>,
    pub curr_ind: usize,
    pub depth: u8,
    pub newboard_rank: u8,
    pub newboard_layer: u8,
    pub newboard_in_a_row: u8,
    pub newboard_numteams: u8,
    pub curr_logfile_path: String,
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub my_team_id: u8,
    pub check_thread_kill: Arc<AtomicBool>,
    pub check_thread_check: Arc<AtomicBool>,
    conn_game_id: usize,
    curr_ip: String,
    curr_err_msg: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        App {
            game: Game::default_game(),
            num_moves: Arc::new(AtomicUsize::new(0)),
            curr_ind: 0,
            depth: 3,
            newboard_in_a_row: 3,
            newboard_layer: 3,
            newboard_rank: 2,
            newboard_numteams: 2,
            curr_logfile_path: String::new(),
            stream: None,
            my_team_id: 0,
            check_thread_kill: Arc::new(AtomicBool::new(false)),
            check_thread_check: Arc::new(AtomicBool::new(true)),
            curr_ip: String::from(ADDR),
            conn_game_id: 0,
            curr_err_msg: String::new(),
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
                    } //else if input.key_pressed(UNDO_KEY)
                    //     && let Some(undo) = self.game.prev_moves.pop()
                    // {
                    //     self.game.board.undo_at(undo);
                    //     self.game.curr_team = if self.game.curr_team == 0 {
                    //         (self.game.teams.len() - 1) as u8
                    //     } else {
                    //         self.game.curr_team - 1
                    //     };
                    //     self.game.correct_box = if let Some(prev) = self.game.prev_moves.last() {
                    //         self.game.board.get_next_correct_move_box(*prev).unwrap()
                    //     } else {
                    //         0
                    //     }
                    // }
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
        if self.game.board.has_ancestor(ind, self.game.correct_box)
            && self.game.curr_team == self.my_team_id
        {
            self.do_move(ind);
            self.game.move_at(ind);
            self.num_moves.fetch_add(1, ORDER);
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
    ///returns true if the connection was set up properly
    pub fn send_join_game(&mut self, game_id: usize) {
        self.terminate();
        let mut stream = if let Ok(x) = TcpStream::connect(&self.curr_ip) {
            self.curr_err_msg = String::from("Connected!");
            x
        } else {
            self.curr_err_msg = String::from("Connection failed!");
            return;
        };
        let mut v = Vec::new();
        v.extend_from_slice(&PASSWORD.to_le_bytes());
        v.push(JOINGAME);
        v.extend_from_slice(&game_id.to_le_bytes());
        stream.write(v.as_slice()).unwrap();
        let mut exists_buffer = [0; 1];
        while stream.read(&mut exists_buffer).unwrap() == 0 {}
        match exists_buffer[0] {
            GAME_EXISTS => {
                self.stream = Some(Arc::new(Mutex::new(stream)));
                self.get_game_data();
                self.restart_check_thread();
                self.curr_err_msg = format!("Connected to game #{}", game_id);
            }
            NO_SUCH_GAME => {
                self.curr_err_msg = String::from("No game found with that ID!");
            }
            _ => {
                self.curr_err_msg = String::from("Game sent invalid signal!");
            }
        }
    }
    pub fn send_make_new_game(&mut self, layers: u8, rank: u8, num_teams: u8, in_a_row: u8) {
        self.terminate();
        let mut stream = if let Ok(x) = TcpStream::connect(&self.curr_ip) {
            self.curr_err_msg = String::from("Connected!");
            x
        } else {
            self.curr_err_msg = String::from("Connection failed!");
            return;
        };
        let mut v = Vec::new();
        v.extend_from_slice(&PASSWORD.to_le_bytes());
        v.push(NEWGAME);
        v.push(layers.to_le());
        v.push(rank.to_le());
        v.push(in_a_row.to_le());
        v.push(num_teams.to_le());
        stream.write(v.as_slice()).unwrap();
        let mut id_buf = [0; 8];
        while stream.read(&mut id_buf).unwrap() == 0 {}
        let gid = usize::from_le_bytes(id_buf);
        self.conn_game_id = gid;
        let board = Board::new(rank, layers, in_a_row);
        self.game = Game::new(board, default_teams(u8::from_le(num_teams)), 0, vec![], 0);
        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.restart_check_thread();
        self.curr_err_msg = format!("Game with ID {gid} created and joined!");
    }
    pub fn restart_check_thread(&mut self) {
        self.check_thread_kill.swap(true, ORDER);
        self.check_thread_kill = Arc::new(AtomicBool::new(false));
        let check_check = self.check_thread_check.clone();
        let second_stream = self.stream.as_ref().unwrap().clone();
        let check_kill = self.check_thread_kill.clone();
        let num_moves2 = self.num_moves.clone();
        thread::spawn(move || {
            loop {
                if check_kill.fetch_and(true, ORDER) == true {
                    break;
                } else {
                    let val = check(
                        &mut second_stream.lock().unwrap(),
                        num_moves2.fetch_add(0, ORDER),
                    );
                    check_check.swap(val, ORDER);
                }
                sleep(Duration::from_millis(100));
            }
        });
    }
    pub fn get_game_data(&mut self) {
        if let Some(ref str) = self.stream {
            let mut stream = str.lock().unwrap();
            let mut header = [0; 5];
            while stream.read(&mut header).unwrap() == 0 {}
            let board = Board::new(
                u8::from_le(header[1]),
                u8::from_le(header[0]),
                u8::from_le(header[2]),
            );
            self.game = Game::new(board, default_teams(u8::from_le(header[3])), 0, vec![], 0);
            let mut length_buffer = [0; 8];
            let mut move_buffer = [0; 8];
            while stream.read(&mut length_buffer).unwrap() == 0 {}
            let len = usize::from_le_bytes(length_buffer);
            self.num_moves.swap(len, ORDER);
            for _ in 0..len {
                while stream.read(&mut move_buffer).unwrap() == 0 {}
                let mov = usize::from_le_bytes(move_buffer.clone());
                self.game.move_at(mov);
            }
            self.game.curr_team = u8::from_le(header[4]);
        }
    }
    pub fn check(&mut self) -> bool {
        if let Some(ref str) = self.stream {
            check(&mut str.lock().unwrap(), self.num_moves.fetch_add(0, ORDER))
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
            self.num_moves.fetch_add(len, ORDER);
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
        self.stream = None;
    }
    pub fn catch_up(&mut self) {
        let check = self.check_thread_check.fetch_and(true, ORDER);
        if !check {
            self.read_moves();
            self.check_thread_check.swap(true, ORDER);
        }
    }
}

pub fn check(stream: &mut TcpStream, len: usize) -> bool {
    let mut buf = [0; 1];
    let mut v = Vec::new();
    v.push(CHECK);
    v.extend_from_slice(&len.to_le_bytes());
    if let Err(_) = stream.write(v.as_slice()) {
        return true;
    };
    while if let Ok(x) = stream.read(&mut buf) {
        x == 0
    } else {
        return true;
    } {}
    let val = u8::from_le(buf[0]);
    if val == 0 { false } else { true }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let screen_size = ui.available_rect_before_wrap();
            ui.label("In a Row");
            ui.add(Slider::new(&mut self.newboard_in_a_row, 2..=8));
            ui.label("Layer");
            ui.add(Slider::new(&mut self.newboard_layer, 2..=8));
            ui.label("Rank");
            ui.add(Slider::new(&mut self.newboard_rank, 1..=5));
            ui.label("Number of Teams");
            ui.add(Slider::new(&mut self.newboard_numteams, 2..=4));
            // if ui.button("Generate New Board").clicked() {
            //     self.game.board = Board::new(
            //         self.newboard_rank,
            //         self.newboard_layer,
            //         self.newboard_in_a_row,
            //     );
            //     self.game.curr_team = 0;
            //     self.game.correct_box = 0;
            //     self.game.prev_moves = vec![];
            //     self.curr_ind = 0;
            //     self.depth = self.newboard_rank.min(5);
            // }
            if ui.button("Make New Game").clicked() {
                self.send_make_new_game(
                    self.newboard_layer,
                    self.newboard_rank,
                    self.newboard_numteams,
                    self.newboard_in_a_row,
                );
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
            ui.label("Team Number");
            ui.add(Slider::new(
                &mut self.my_team_id,
                0..=(self.game.teams.len() - 1) as u8,
            ));
            ui.label("Server Address");
            ui.add(TextEdit::singleline(&mut self.curr_ip));
            ui.label("Game ID");
            ui.add(DragValue::new(&mut self.conn_game_id));
            if ui.button("Join Game").clicked() {
                self.send_join_game(self.conn_game_id);
            }
            if ui.button("Disconnect").clicked() {
                self.terminate();
            }
            let rect_size = Vec2::splat(screen_size.size().y.min(screen_size.size().x));
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
            // if ui.button("CHECK").clicked() {
            //     dbg!(self.check());
            // }
            // if ui.button("READ").clicked() {
            //     self.read_moves();
            // }
            // if ui.button("MOVE").clicked() {
            //     self.do_move(self.val);
            // }
            // if ui.button("TERMINATE").clicked() {
            //     self.terminate();
            // }
            // ui.add(Slider::new(&mut self.val, 0..=100));
            ui.label(&self.curr_err_msg);
            //self.check_connection();
        });
        ctx.request_repaint();
    }
}
