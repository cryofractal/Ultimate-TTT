#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::Path};

use crate::{app::Game, board::Board, team::default_teams};

impl Game {
    fn to_logfile_string(&self) -> String {
        let mut str = String::new();
        for val in [
            self.board.layer,
            self.board.rank,
            self.board.in_a_row,
            self.curr_team,
        ] {
            str += &format!("{},", val);
        }
        for data in &self.prev_moves {
            str += &format!("{},", data);
        }
        str.remove(str.len() - 1);
        str
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_to_file(&self, path: &Path) {
        let data = self.to_logfile_string();
        let _ = fs::write(path, data);
    }
    fn from_logfile_string(str: String) -> Option<Self> {
        let vals = str
            .split(",")
            .map(|x| x.parse::<usize>().ok())
            .collect::<Vec<Option<usize>>>();
        let mut app = Self {
            board: Board::new(
                (*vals.get(1)?)? as u8,
                (*vals.get(0)?)? as u8,
                (*vals.get(2)?)? as u8,
            ),
            teams: default_teams(),
            curr_ind: 0,
            curr_team: (*vals.get(3)?)? as u8,
            prev_moves: Vec::with_capacity(vals.len()),
            correct_box: 0,
        };
        for ind in 4..vals.len() {
            let mov = vals[ind]?;
            app.board
                .move_at_index(mov, ((ind - 4) % app.teams.len()) as u8);
            app.prev_moves.push(mov);
        }
        if vals.len() > 4
            && let Some(v) = vals.last()
            && let Some(nextmovebox) = app.board.get_next_correct_move_box((*v)?)
        {
            app.correct_box = nextmovebox;
        }
        Some(app)
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_file(path: &Path) -> Option<Self> {
        let string = String::from_utf8(fs::read(path).ok()?).ok()?;
        Self::from_logfile_string(string)
    }
}
