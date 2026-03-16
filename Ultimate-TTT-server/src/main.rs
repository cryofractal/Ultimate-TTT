use std::{
    env,
    fs::{self, OpenOptions},
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
pub const PORT: &str = ":52525";
//pub const ADDR: &str = "68.183.121.208:52525";
pub const PASSWORD: usize = 182309128390812;
pub const NEWGAME: u8 = 0_u8.to_le();
pub const JOINGAME: u8 = 1_u8.to_le();
pub const NO_SUCH_GAME: u8 = 0_u8.to_le();
pub const GAME_EXISTS: u8 = 1_u8.to_le();
pub const SAVE_DELAY: u64 = 10;

pub struct Game {
    id: usize,
    moves: Vec<usize>,
    layers: u8,
    rank: u8,
    in_a_row: u8,
    curr_team: u8,
    num_teams: u8,
}

const EXTENSION: &str = "game";
const PREFIX: &str = "TTTGAME";
const NUM_GAMES_FILENAME: &str = "./Games/NUMGAMES.txt";
impl Game {
    pub fn load(id: usize) -> std::io::Result<Self> {
        let filename = format!("./Games/{}-{}.{}", PREFIX, id, EXTENSION);
        let data = fs::read(filename)?;
        let e = Error::new(
            std::io::ErrorKind::InvalidInput,
            "Error loading from file: File too short or data is improper!",
        );
        if data.len() < 5 || data.len() % 8 != 5 {
            return Err(e);
        }
        let mut g = Self {
            id,
            moves: vec![],
            layers: u8::from_le(data[0]),
            rank: u8::from_le(data[1]),
            in_a_row: u8::from_le(data[2]),
            curr_team: u8::from_le(data[3]),
            num_teams: u8::from_le(data[4]),
        };
        let mut ind = 5;
        while ind + 8 <= data.len() {
            let nextmove = usize::from_le_bytes(*data[ind..(ind + 8)].as_array().unwrap());
            g.moves.push(nextmove);
            ind += 8;
        }
        Ok(g)
    }
    pub fn save(&self) -> std::io::Result<()> {
        let filename = format!("./Games/{}-{}.{}", PREFIX, self.id, EXTENSION);
        let mut buf = Vec::new();
        buf.extend([
            self.layers.to_le(),
            self.rank.to_le(),
            self.in_a_row.to_le(),
            self.curr_team.to_le(),
            self.num_teams.to_le(),
        ]);
        for mov in &self.moves {
            buf.extend(mov.to_le_bytes());
        }
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .open(filename)?;
        f.write_all(buf.as_slice())?;
        Ok(())
    }
    pub fn new(layer: u8, rank: u8, in_a_row: u8, num_teams: u8, id: usize) -> Self {
        Self {
            moves: vec![],
            layers: layer,
            rank,
            in_a_row,
            curr_team: 0,
            num_teams,
            id,
        }
    }
    pub fn move_at(&mut self, ind: usize) {
        self.moves.push(ind);
        self.curr_team = (self.curr_team + 1) % self.num_teams;
    }
    pub fn write_data(&self, stream: &mut TcpStream) -> std::io::Result<()> {
        let header = [
            self.layers.to_le(),
            self.rank.to_le(),
            self.in_a_row.to_le(),
            self.num_teams.to_le(),
            self.curr_team.to_le(),
        ];
        let len = self.moves.len().to_le_bytes();
        let mut v = Vec::new();
        v.push(GAME_EXISTS);
        v.extend_from_slice(&header);
        v.extend_from_slice(&len);
        for mov in &self.moves {
            v.extend_from_slice(&mov.to_le_bytes());
        }
        stream.write(v.as_slice())?;
        Ok(())
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
    // pub fn new_test() -> Self {
    //     Self {
    //         moves: vec![],
    //         layers: 3,
    //         rank: 2,
    //         in_a_row: 3,
    //         curr_team: 0,
    //         num_teams: 2,
    //     }
    // }
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
    pub fn save(&self) -> std::io::Result<()> {
        OpenOptions::new()
            .truncate(true)
            .create(true)
            .open(NUM_GAMES_FILENAME)?
            .write(&self.games.len().to_le_bytes())?;
        for game in &self.games {
            game.lock().unwrap().save()?;
        }
        Ok(())
    }
    pub fn load() -> std::io::Result<Self> {
        let len = usize::from_le_bytes(*fs::read(NUM_GAMES_FILENAME)?[0..8].as_array().unwrap());
        let mut new_state = Self {
            games: Vec::with_capacity(len),
        };
        for i in 0..len {
            new_state.games.push(Arc::new(Mutex::new(Game::load(i)?)));
        }
        Ok(new_state)
    }
    pub fn update(&mut self, mut stream: TcpStream) -> std::io::Result<()> {
        let mut buf = [0; 8];
        while stream.read(&mut buf)? == 0 {}
        let pass = usize::from_le_bytes(*buf[0..8].as_array().unwrap());
        if pass == PASSWORD {
            let mut commbuff = [0; 1];
            while stream.read(&mut commbuff)? == 0 {}
            let comm_byte = commbuff[0];
            match comm_byte {
                NEWGAME => {
                    let mut header_buff = [0; 4];
                    while stream.read(&mut header_buff)? == 0 {}
                    let (layer, rank, in_a_row, num_teams) = (
                        u8::from_le(header_buff[0]),
                        u8::from_le(header_buff[1]),
                        u8::from_le(header_buff[2]),
                        u8::from_le(header_buff[3]),
                    );

                    let id = self.games.len();
                    self.games.push(Arc::new(Mutex::new(Game::new(
                        layer, rank, in_a_row, num_teams, id,
                    ))));
                    while stream.write(&id.to_le_bytes())? == 0 {}
                    let mut conn = Connection {
                        stream,
                        game: self.games[id].clone(),
                        team_id: u8::from_le(buf[0]),
                    };
                    thread::spawn(move || {
                        loop {
                            let update_result = conn.update();
                            match update_result {
                                Ok(x) => {
                                    if x {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    println!("Error occured: {e}");
                                    break;
                                }
                            }
                        }
                    });
                    Ok(())
                }
                JOINGAME => {
                    while stream.read(&mut buf)? == 0 {}
                    let game_id = usize::from_le_bytes(buf);
                    if game_id >= self.games.len() {
                        stream.write(&[NO_SUCH_GAME])?;
                        return Ok(());
                    }
                    self.games[game_id]
                        .lock()
                        .unwrap()
                        .write_data(&mut stream)?;
                    let mut conn = Connection {
                        stream,
                        game: self.games[game_id].clone(),
                        team_id: u8::from_le(buf[0]),
                    };
                    thread::spawn(move || {
                        loop {
                            let update_result = conn.update();
                            match update_result {
                                Ok(x) => {
                                    if x {
                                        break;
                                    }
                                }
                                Err(e) => {
                                    println!("Error occured: {e}");
                                    break;
                                }
                            }
                        }
                    });
                    Ok(())
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }
}

impl Connection {
    pub fn read_comm(&mut self) -> std::io::Result<Option<Command>> {
        let mut buf = [0; 9];
        while self.stream.read(&mut buf)? == 0 {}
        let comm = Command::from_byte(
            u8::from_le(buf[0]),
            usize::from_le_bytes(*buf[1..].as_array().unwrap()),
        );
        Ok(comm)
    }
    pub fn write_check_result(&mut self, check: bool) -> std::io::Result<()> {
        let val = if check { 1_u8.to_le() } else { 0_u8.to_le() };
        self.stream.write(&[val])?;
        Ok(())
    }
    //return true if the connection is terminated.
    pub fn update(&mut self) -> std::io::Result<bool> {
        let comm = self.read_comm()?.ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid command byte!",
        ))?;
        match comm {
            Command::Check(ind) => {
                let check = self.game.lock().unwrap().check(ind);
                self.write_check_result(check)?;
            }
            Command::ReadMove(ind) => {
                let game = self.game.lock().unwrap();
                game.write_moves_from(ind, &mut self.stream);
            }
            Command::DoMove(ind) => {
                self.game.lock().unwrap().move_at(ind);
            }
            Command::Terminate(_) => {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let ip = args.get(1).expect("Please pass the IP as an argument");
    let addr = ip.clone() + PORT;
    let state = Arc::new(Mutex::new(if let Ok(s) = State::load() {
        s
    } else {
        State::new()
    }));
    let state2 = state.clone();
    thread::spawn(move || {
        sleep(Duration::from_secs(1800));
        state2.lock().unwrap().save().unwrap();
    });
    loop {
        let listener = TcpListener::bind(&addr).unwrap();
        let (stream, _addr) = listener.accept().unwrap();
        if let Err(x) = state.lock().unwrap().update(stream) {
            println!("State error occured: {x}")
        };
    }
}
