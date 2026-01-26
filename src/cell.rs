use crate::team::*;

pub struct Cell {
    // Which layer the cell is in. 1 is the outermost layer.
    rank: i8,
    // Which position the cell is in in its parent cell. numbered 1-9 from top left to bottom right
    pos: i8,
    // Cells contained within the larger cell
    children: Vec<Cell>,
    // Current status of play of the cell
    state: CellState,
}

enum CellState {
    Owned(Team),
    Empty,
    Contested,
}