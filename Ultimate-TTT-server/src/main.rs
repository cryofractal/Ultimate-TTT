use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};
pub const ADDR: &str = "100.67.217.200:52525";
pub const PASSWORD: usize = 182309128390812;

pub struct Game {
    moves: Vec<usize>,
    layers: u8,
    rank: u8,
    in_a_row: u8,
    curr_team: u8,
    num_teams: u8,
}

impl Game {
    pub fn move_at(&mut self, ind: usize) {
        self.moves.push(ind);
        self.curr_team = (self.curr_team + 1) % self.num_teams;
    }
    pub fn write_data(&self, stream: &mut TcpStream) {
        let header = [
            self.layers.to_le(),
            self.rank.to_le(),
            self.in_a_row.to_le(),
            self.curr_team.to_le(),
        ];
        let len = self.moves.len().to_le_bytes();
        let mut v = Vec::new();
        v.extend_from_slice(&header);
        v.extend_from_slice(&len);
        for mov in &self.moves {
            v.extend_from_slice(&mov.to_le_bytes());
        }
        stream.write(v.as_slice()).unwrap();
    }
    //also writes the current team id
    pub fn write_moves_from(&self, start: usize, stream: &mut TcpStream) {
        let diff = if start > self.moves.len() {
            0
        } else {
            self.moves.len() - start
        };
        let mut v = Vec::new();
        v.push(self.curr_team.to_le());
        v.extend_from_slice(&diff.to_le_bytes());
        for mov in &self.moves[start..self.moves.len()] {
            v.extend_from_slice(&mov.to_le_bytes());
        }
        stream.write(v.as_slice()).unwrap();
    }
    pub fn new_test() -> Self {
        Self {
            moves: vec![],
            layers: 3,
            rank: 2,
            in_a_row: 3,
            curr_team: 0,
            num_teams: 2,
        }
    }
    pub fn check(&self, ind: usize) -> bool {
        ind == self.moves.len()
    }
}
#[derive(PartialEq)]
pub enum Command {
    Check(usize),     //0
    ReadMove(usize),  //1
    DoMove(usize),    //2
    Terminate(usize), //3
}

impl Command {
    pub fn from_byte(byte: u8, val: usize) -> Option<Self> {
        Some(match byte {
            0 => Self::Check(val),
            1 => Self::ReadMove(val),
            2 => Self::DoMove(val),
            3 => Self::Terminate(val),
            _ => {
                return None;
            }
        })
    }
}

pub struct Connection {
    pub stream: TcpStream,
    pub game: Arc<Mutex<Game>>,
    pub team_id: u8,
}
pub struct State {
    games: Vec<Arc<Mutex<Game>>>,
}

impl State {
    pub fn new() -> Self {
        Self { games: vec![] }
    }
    pub fn update(&mut self) {
        let listener = TcpListener::bind(ADDR).unwrap();
        let (mut stream, _addr) = listener.accept().unwrap();
        let mut buf = [0; 16];
        while stream.read(&mut buf).unwrap() == 0 {}
        let pass = usize::from_le_bytes(*buf[0..8].as_array().unwrap());
        if pass == PASSWORD {
            let game_id = usize::from_le_bytes(*buf[8..].as_array().unwrap());
            self.games[game_id].lock().unwrap().write_data(&mut stream);
            let mut conn = Connection {
                stream,
                game: self.games[game_id].clone(),
                team_id: u8::from_le(buf[0]),
            };
            thread::spawn(move || while !conn.update() {});
        }
        // } else {
        //     for connection in &mut self.connections {
        //         let game = &mut self.games[connection.game_id];
        //         let handle = thread::spawn(|| connection.update())
        //         let comm = connection.update();
        //         match comm {
        //             Command::MakeNewGame(_) => todo!(),
        //             Command::ConnectToExistingGame(_) => todo!(),
        //             Command::DoMove(ind) => {
        //                 let game = &mut self.games[connection.game_id];
        //                 if game.curr_team == connection.team_id {
        //                     game.move_at(ind);
        //                 }
        //             }
        //         }
        // }
        //}
    }
}

impl Connection {
    pub fn read_comm(&mut self) -> Command {
        let mut buf = [0; 9];
        while self.stream.read(&mut buf).unwrap() == 0 {}
        let comm = Command::from_byte(
            u8::from_le(buf[0]),
            usize::from_le_bytes(*buf[1..].as_array().unwrap()),
        );
        comm.unwrap()
    }
    pub fn write_check_result(&mut self, check: bool) {
        let val = if check { 1_u8.to_le() } else { 0_u8.to_le() };
        self.stream.write(&[val]).unwrap();
    }
    //return true if the connection is terminated.
    pub fn update(&mut self) -> bool {
        let comm = self.read_comm();
        match comm {
            Command::Check(ind) => {
                let check = self.game.lock().unwrap().check(ind);
                self.write_check_result(check);
            }
            Command::ReadMove(ind) => {
                let game = self.game.lock().unwrap();
                game.write_moves_from(ind, &mut self.stream);
            }
            Command::DoMove(ind) => {
                self.game.lock().unwrap().move_at(ind);
            }
            Command::Terminate(_) => {
                return true;
            }
        }
        false
    }
}

fn main() {
    let mut state = State {
        games: vec![Arc::new(Mutex::new(Game::new_test()))],
    };
    loop {
        state.update();
    }
}
