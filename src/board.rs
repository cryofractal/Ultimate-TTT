type Index = u8;

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
    pub state_array: Vec<CellState>,
    // Previous path played (in order to undo)
    pub prev_path: Vec<Coord>,
}

impl Board {
    pub fn new(rank: u8, layer: u8, dim: u8) -> Self {
        Board {
            rank: rank,
            layer: layer,
            dim: dim,
            state_array: [],
            prev_path: Vec::new(),
        }
    }
    pub fn get(&self, path: &[Coord]) -> &CellState {
        self.state_array[self.pathToIndex(path)]
    }

    pub fn children(&self, path: Vec<Coord>) -> &[CellState] {
        //check if the rank is
        if path.len() >= self.rank {
            []
        } else {
            let ind = self.pathToIndex(path);
            state_array[pow(self.layer, self.dim) * ind..pow(self.layer, self.dim) * (ind + 1)]
        }
    }

    pub fn pathToIndex(&self, path: Vec<Coord>) -> Index {
        //check that the nesting is no deeper than rank
        if path.len() > self.rank {
            todo!();
        }

        let ind: Index = 0;
        for i in 0..path.len() {
            ind = pow(self.layer, self.dim) * ind + self.coordToRelativeIndex(path[i]);
        }
        ind
    }

    pub fn coordToRelativeIndex(&self, pos: Coord) -> Index {
        //check that the size of the coord is the same as the dimension
        if pos.len() != self.dim {
            todo!();
        }
        let ind: Index = 0;
        for i in 0..pos.len() {
            ind = pow(self.layer, self.dim) * ind + pos[i];
        }
    }
}
