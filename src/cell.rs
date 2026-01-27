use std::{
    collections::HashMap,
    ops::{Div, Sub},
    rc::Rc,
};

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

fn sub_coords(lhs: &Coord, rhs: &Coord) -> Vec<i16> {
    if lhs.coord.len() != rhs.coord.len() {
        panic!("Trying to subtract coords of different sizes")
    }
    let mut diff: Vec<i16> = Vec::new();
    for i in 0..lhs.coord.len() {
        diff.push((lhs.coord[i]) as i16 - rhs.coord[i] as i16);
    }
    diff
}

const CELL_NUM: usize = 9;
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
            Self::captured_set(
                self.children
                    .iter()
                    .filter(|x| x.1.state == CellState::Owned(team_id))
                    .map(|x| x.0)
                    .collect(),
            )
        }
    }
    pub fn captured_set(set: Vec<&Coord>) -> bool {
        let use_subset_alg = true;
        if use_subset_alg {
            //Use subsets alg
            dbg!(subsets_of_size(set, 3))
                .iter()
                .any(|x| Self::captured_subset(x.clone()))
        } else {
            //Use lines alg
            todo!()
        }
    }
    pub fn captured_subset(mut set: Vec<&Coord>) -> bool {
        set.sort_by_key(|x| x.coord.clone());
        let base_diff = sub_coords(set[0], set[1]);
        for i in 2..set.len() {
            if sub_coords(set[i - 1], set[i]) != base_diff {
                return false;
            }
        }
        true
    }

    // O(l2^d) l=layers, d=dimensions
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
// O(l^d choose l) l=layers, d=dimensions
fn subsets_of_size(set: Vec<&Coord>, size: usize) -> Vec<Vec<&Coord>> {
    if set.len() < size {
        Vec::new()
    } else if set.len() == size {
        vec![set]
    } else if size == 0 {
        vec![Vec::new()]
    } else {
        let mut first_included = subsets_of_size(Vec::from(&set[1..]), size);
        let first_excluded: Vec<Vec<&Coord>> = subsets_of_size(Vec::from(&set[1..]), size - 1)
            .iter()
            .map(|x| {
                let mut vec = vec![set[0]];
                vec.extend_from_slice(x);
                vec
            })
            .collect();
        first_included.extend(first_excluded);
        first_included
    }
}
