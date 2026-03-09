use crate::board::{Board, Coord};

///The difference of two Coords as a Vec<i16>
fn sub_coords(lhs: Coord, rhs: Coord) -> Vec<i16> {
    let mut diff: Vec<i16> = Vec::new();
    diff.push((lhs.x) as i16 - rhs.x as i16);
    diff.push((lhs.y) as i16 - rhs.y as i16);
    diff
}

impl Board {
    pub fn is_solved(&self, tiles: &[u8], team_id: u8, pos: Coord) -> bool {
        captured_set(
            (0..self.grid_num())
                .filter(|x| tiles[*x] == team_id)
                .map(|x| self.rel_index_to_coord(x))
                .collect(),
            pos,
            self.in_a_row as usize,
        )
    }
}

///Returns whether there exists a winning line within [`set`]
fn captured_set(set: Vec<Coord>, pos: Coord, in_a_row: usize) -> bool {
    let use_subset_alg = true;
    if use_subset_alg {
        //Use subsets
        // Worst case: O(l^d choose l) l=layers, d=dimensions
        //if any subset of size 3 is a winning line
        subsets_of_size_containing(set, in_a_row, vec![pos])
            .iter()
            .any(|x| captured_subset(x.clone()))
    } else {
        //Use lines alg
        // O(l2^d) l=layers, d=dimensions
        todo!()
    }
}

/// Returns the set of all subsets of [`set`] with cardinality [`size`] as a Vec<Vec<>>
fn subsets_of_size(set: Vec<Coord>, size: usize) -> Vec<Vec<Coord>> {
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
        let first_included: Vec<Vec<Coord>> = subsets_of_size(Vec::from(&set[1..]), size - 1)
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

/// Returns the set of all subsets of [`set`] with cardinality [`size`] that contain all the elements in [`base`] as a Vec<Vec<>>
fn subsets_of_size_containing(set: Vec<Coord>, size: usize, base: Vec<Coord>) -> Vec<Vec<Coord>> {
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
        //The set of subsets with cardinality `size-base.len()` from set, each with all of base elements
        subsets_of_size(set, size - base.len())
            .iter()
            .map(|x| {
                let mut vec = base.clone();
                vec.extend_from_slice(x);
                vec
            })
            .collect()
    }
}

/// Returns whether all of [`set`] is in the same line
fn captured_subset(mut set: Vec<Coord>) -> bool {
    //Sorts the set lexigraphically so the differences from the next step should be the same
    set.sort_by_key(|c| [c.x, c.y]);
    let base_diff = sub_coords(set[0], set[1]);
    //Makes sure every adjacent pair is the same difference as the base difference
    for i in 2..set.len() {
        if sub_coords(set[i - 1], set[i]) != base_diff {
            return false;
        }
    }
    true
}
