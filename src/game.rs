use crate::*;

pub struct Game {
    pub board: Board,
    pub teams: Vec<Team>,
    pub curr_team: u8,
    pub prev_moves: Vec<usize>,
    pub correct_box: usize,
}

impl Game {
    pub fn new(
        board: Board,
        teams: Vec<Team>,
        curr_team: u8,
        prev_moves: Vec<usize>,
        correct_box: usize,
    ) -> Self {
        Game {
            board,
            teams,
            curr_team,
            prev_moves,
            correct_box,
        }
    }
    //Generate Default Game
    pub fn default_game() -> Self {
        Game::new(Board::new(2, 3, 3), default_teams(), 0, vec![], 0)
    }
    //returns if a move happened
    pub fn move_at(&mut self, ind: usize) {
        self.board.move_at_index(ind, self.curr_team);
        self.correct_box = self.board.get_next_correct_move_box(ind).unwrap();
        self.prev_moves.push(ind);
        self.curr_team = (self.curr_team + 1) % self.teams.len() as u8;
        self.curr_ind = 0;
        if direct {
            self.write_to_connection(ind);
        }
    }
}
