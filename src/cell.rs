use std::{collections::HashMap, rc::Rc};

type Index = u8;

use const_vec::ConstVec;
use itertools::Itertools;

use crate::team::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Coord {
    pub coord: Vec<Index>,
}

#[macro_export]
macro_rules! coord {
    ($($x:expr),*) => {
        {
            let mut vect = Vec::new();
            $(vect.push($x);)*
            Coord {coord: vect}
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    // Which layer the cell is in. 0 is the outermost layer.
    pub rank: u8,
    // Cells contained within the larger cell
    pub children: HashMap<Coord, Cell>,
    // Current status of play of the cell
    pub state: CellState,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CellState {
    Owned(u8),
    Empty,
    Contested,
}

impl CellState {
    pub fn is_nonempty(&self) -> bool {
        match self {
            CellState::Owned(_) => true,
            CellState::Empty => false,
            CellState::Contested => true,
        }
    }
    pub fn owned(&self) -> Option<u8> {
        if let CellState::Owned(t) = self {
            Some(*t)
        } else {
            None
        }
    }
}

impl Cell {
    pub fn new(rank: u8) -> Self {
        Cell {
            rank: rank,
            children: HashMap::new(),
            state: CellState::Empty,
        }
    }
    pub fn get(&self, path: &[Coord]) -> &Cell {
        if path.len() > 0 {
            self.children.get(&path[0]).unwrap().get(&path[1..])
        } else {
            self
        }
    }
    pub fn update(&mut self, path: &[Coord], team_id: u8) -> bool {
        if path.len() > 0 {
            if !self
                .children
                .get_mut(&path[0])
                .unwrap()
                .update(&path[1..], team_id)
            {
                return false;
            }
        } else {
            self.state = CellState::Owned(team_id);
            return true;
        }
        let check = self.check(team_id);
        if check != self.state {
            self.state = check;
            true
        } else {
            false
        }
    }
    pub fn check(&self, team_id: u8) -> CellState {
        if self.children.values().any(|x| x.state.is_nonempty()) {
            if self.captured(team_id) {
                CellState::Owned(team_id)
            } else {
                CellState::Contested
            }
        } else {
            CellState::Empty
        }
    }
    pub fn captured(&self, team_id: u8) -> bool {
        if self.children.len() != 9 {
            todo!()
        } else {
            let mut set = HashMap::new();
            self.children
                .iter()
                .filter(|x| x.1.state == CellState::Owned(team_id))
                .map(|x| x.0)
                .for_each(|x| {
                    set.insert(x, ());
                });
            Self::captured_set(set)
        }
    }
    pub fn captured_set(set: HashMap<&Coord, ()>) -> bool {
        for row in Self::three_in_a_rows() {
            if row.iter().all(|x| set.contains_key(x)) {
                return true;
            }
        }
        false
    }
    fn three_in_a_rows() -> Vec<[Coord; 3]> {
        vec![
            [coord![0, 0], coord![0, 1], coord![0, 2]],
            [coord![0, 0], coord![1, 0], coord![2, 0]],
            [coord![0, 0], coord![1, 1], coord![2, 2]],
            [coord![2, 2], coord![2, 1], coord![2, 0]],
            [coord![2, 2], coord![1, 2], coord![0, 2]],
            [coord![0, 2], coord![1, 1], coord![0, 2]],
            [coord![1, 0], coord![1, 1], coord![1, 2]],
            [coord![0, 1], coord![1, 1], coord![2, 1]],
        ]
    }
}
