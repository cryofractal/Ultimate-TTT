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

///The difference of two Coords as a Vec<i16>
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
    ///Updates the cell at path to be caputed by the team with [`team_id`] by recursively telling
    /// the current cell's child with coords `path[0]` to update with path `path[1..]`
    /// and returns whether or not the specific cell [`self`] had its cell state changed
    pub fn update(&mut self, path: &[Coord], team_id: u8) -> bool {
        if path.len() > 0 {
            //Path has layers left -> update the child cell at the next level of coord
            //                      with the current outermost coord removed so it can propperly recurse
            //                      and return false if no update made
            if !self
                .children
                .get_mut(&path[0])
                .unwrap()
                .update(&path[1..], team_id)
            {
                //Short curcuit if no resulting change and propogate the lack of change
                return false;
            }
        } else {
            //Path empty means this is the lowest level -> directly set the team to own it and return that it has changed
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
    ///Recalulates the cell state given that the team with `team_id` has just moved
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
    ///Returns whether a winning line has been added to the cell for the team with [`team_id`]
    /// from them having captured the cell at [`coord`]
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
    ///Returns whether there exists a winning line within [`set`]
    pub fn captured_set(set: Vec<&Coord>) -> bool {
        let use_subset_alg = true;
        if use_subset_alg {
            //Use subsets
            // Worst case: O(l^d choose l) l=layers, d=dimensions
            //if any subset of size 3 is a winning line
            subsets_of_size(set, 3)
                .iter()
                .any(|x| captured_subset(x.clone()))
        } else {
            //Use lines alg
            // O(l2^d) l=layers, d=dimensions
            todo!()
        }
    }

    /// Returns the set of potential lines on a 3x3 board
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
/// Returns the set of all subsets of [`set`] with cardinality [`size`] as a Vec<Vec<>>
fn subsets_of_size(set: Vec<&Coord>, size: usize) -> Vec<Vec<&Coord>> {
    if set.len() < size {
        //There are no subsets of a size greater than the set
        Vec::new() //The empty set
    } else if set.len() == size {
        //Base case 1
        //The set is the only subset with the same size as the set
        vec![set] //The set of the set
    } else if size == 0 {
        //Base case 2
        //The empty set is the only set of size 0 and is a subset of any set
        vec![Vec::new()] //The set of the empty set
    } else {
        //Recursive step
        //Every subset will either include or exclude the first element.
        //The ones that exclude are just the subsets of the same size of (set - first element)
        let mut first_excluded = subsets_of_size(Vec::from(&set[1..]), size);
        //The ones that contain the first element are going to be the subsets of size-1 of (set - first element)
        //but with the first element added to each set
        let first_included: Vec<Vec<&Coord>> = subsets_of_size(Vec::from(&set[1..]), size - 1)
            .iter()
            .map(|x| {
                let mut vec = vec![set[0]];
                vec.extend_from_slice(x);
                vec
            })
            .collect();
        first_excluded.extend(first_included); //puts the union of the two into first_excluded
        first_excluded
    }
}

/// Returns whether all of [`set`] is in the same line
fn captured_subset(mut set: Vec<&Coord>) -> bool {
    //Sorts the set lexigraphically so the differences from the next step should be the same
    set.sort_by_key(|x| x.coord.clone());
    let base_diff = sub_coords(set[0], set[1]);
    //Makes sure every adjacent pair is the same difference as the base difference
    for i in 2..set.len() {
        if sub_coords(set[i - 1], set[i]) != base_diff {
            return false;
        }
    }
    true
}
