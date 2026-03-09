type Index = u8;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct Coord {
    x: u8,
    y: u8,
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

pub struct Board {
    // The largest amount of nesting on the Board
    pub rank: u8,
    //The number of layers in each cell
    pub layer: u8,
    //The number of dimensions of each cell
    pub dim: u8,
    // Current statuses of play for each cell
    pub state_array: Vec<u8>,
    // Previous path played (in order to undo)
    pub prev_path: Vec<Coord>,
    // The index of the first leaf
    critical_index: usize,
}

impl Board {
    pub fn new(rank: u8, layer: u8, dim: u8) -> Self {
        let grid_num = (layer as usize).pow(dim as u32);
        let num = grid_num.pow(rank as u32) * (grid_num) / (grid_num - 1);
        let cr_index = num - grid_num.pow(rank as u32);
        Board {
            rank: rank,
            layer: layer,
            dim: dim,
            state_array: vec![0; num],
            prev_path: Vec::new(),
            critical_index: cr_index,
        }
    }
    pub fn grid_num(&self) -> usize {
        self.layer as usize * self.layer as usize
    }
    pub fn get(&self, path: &[Coord]) -> u8 {
        self.state_array[self.path_to_index(path)]
    }

    pub fn children(&self, index: usize) -> Option<&[u8]> {
        if index > self.critical_index {
            None
        } else {
            Some(&self.state_array[(self.grid_num() * index)..(self.grid_num() * (index + 1))])
        }
    }

    pub fn path_to_index(&self, path: &[Coord]) -> usize {
        //check that the nesting is no deeper than rank
        if path.len() > self.critical_index {
            panic!("Path is too long!");
        }

        let mut ind = 1;
        for i in 0..path.len() {
            ind = self.grid_num() * ind + self.coord_to_relative_index(path[i]);
        }
        ind
    }

    pub fn coord_to_relative_index(&self, pos: Coord) -> usize {
        (pos.x as usize * self.layer as usize) + pos.y as usize
    }
    pub fn rel_index_to_coord(&self, index: usize) -> Coord {
        Coord {
            x: (index / self.layer as usize) as u8,
            y: (index % self.layer as usize) as u8,
        }
    }
}
