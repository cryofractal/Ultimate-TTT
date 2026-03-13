#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub struct Coord {
    pub x: u8,
    pub y: u8,
}

pub struct Board {
    // Number of squares in a row to solve a cell
    pub in_a_row: u8,
    // The largest amount of nesting on the Board
    pub rank: u8,
    //The number of layers in each cell
    pub layer: u8,
    // Current statuses of play for each cell
    // 255 - CONTESTED
    // 255 > i - OWNED
    pub state_array: Vec<u8>,
    // Previous path played (in order to undo)
    pub prev_path: Vec<Coord>,
    // The index of the first leaf
    critical_index: usize,
}

impl Board {
    pub fn new(rank: u8, layer: u8, in_a_row: u8) -> Self {
        let grid_num = (layer as usize).pow(2);
        let num = grid_num.pow(rank as u32) * (grid_num) / (grid_num - 1);
        let cr_index = num - grid_num.pow(rank as u32);
        Board {
            in_a_row,
            rank,
            layer,
            state_array: vec![255; num],
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
        if index >= self.critical_index {
            None
        } else {
            Some(
                &self.state_array
                    [((self.grid_num() * index) + 1)..=(self.grid_num() * (index + 1))],
            )
        }
    }
    pub fn is_leaf(&self, index: usize) -> bool {
        index >= self.critical_index
    }
    pub fn children_base(&self, index: usize) -> Option<usize> {
        if index >= self.critical_index {
            None
        } else {
            Some((self.grid_num() * index) + 1)
        }
    }
    pub fn parent(&self, index: usize) -> Option<usize> {
        if index == 0 {
            None
        } else {
            Some((index - 1) / self.grid_num())
        }
    }

    pub fn path_to_index(&self, path: &[Coord]) -> usize {
        //check that the nesting is no deeper than rank
        if path.len() > self.critical_index {
            panic!("Path is too long!");
        }

        let mut ind = 0;
        for i in 0..path.len() {
            ind = self.children_base(ind).unwrap() + self.coord_to_relative_index(path[i]);
        }
        ind
    }
    fn objective_index_to_rel_coord(&self, index: usize) -> Option<Coord> {
        Some(self.rel_index_to_coord(index - self.children_base(self.parent(index)?)?))
    }
    pub fn index_to_path(&self, index: usize) -> Vec<Coord> {
        let mut v = vec![];
        let mut curr = index;
        while let Some(next) = self.parent(curr) {
            v.push(self.objective_index_to_rel_coord(curr).unwrap());
            curr = next;
        }
        v.into_iter().rev().collect()
    }
    pub fn coord_to_relative_index(&self, pos: Coord) -> usize {
        (pos.y as usize * self.layer as usize) + pos.x as usize
    }
    pub fn rel_index_to_coord(&self, index: usize) -> Coord {
        Coord {
            x: (index % self.layer as usize) as u8,
            y: (index / self.layer as usize) as u8,
        }
    }
    pub fn update_at(&mut self, index: usize, id: u8, rel_position: usize) -> bool {
        if index >= self.critical_index {
            true
        } else {
            match self.state_array[index] {
                255 => {
                    if self.is_solved(
                        self.children(index).unwrap(),
                        id,
                        self.rel_index_to_coord(rel_position),
                    ) {
                        self.state_array[index] = id;
                        true
                    } else {
                        false
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    pub fn update_ascending(&mut self, start: usize, id: u8) {
        let mut curr = start;
        while let Some(next) = self.parent(curr)
            && self.update_at(next, id, curr - (next * self.grid_num()))
        {
            curr = next;
        }
    }
    pub fn ascend_set_contested(&mut self, start: usize) {
        let mut curr = start;
        while let Some(next) = self.parent(curr) {
            curr = next;
            self.state_array[next] = 255;
        }
    }
    pub fn move_at(&mut self, path: &[Coord], id: u8) {
        self.move_at_index(self.path_to_index(path), id);
    }
    pub fn move_at_index(&mut self, index: usize, id: u8) {
        self.state_array[index] = id;
        self.update_ascending(index, id);
    }
    pub fn undo_at(&mut self, index: usize) {
        self.state_array[index] = 255;
        self.ascend_set_contested(index);
    }
    pub fn has_ancestor(&self, index: usize, ancestor: usize) -> bool {
        let mut curr = index;
        while let Some(next) = self.parent(curr)
            && next >= ancestor
        {
            curr = next;
            if curr == ancestor {
                return true;
            }
        }
        false
    }
    fn get_first_past_captured(&self, start: usize) -> Option<usize> {
        let mut curr = start;
        let mut curr_return = Some(start);
        while let Some(next) = self.parent(curr) {
            if self.state_array[curr] < 255 {
                curr_return = self.parent(curr);
            }
            curr = next;
        }
        curr_return
    }
    pub fn get_next_correct_move_box(&self, index: usize) -> Option<usize> {
        let path = self.index_to_path(index);
        let start = self.path_to_index(&path[1..]);
        self.get_first_past_captured(start)
    }
}
