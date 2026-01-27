use crate::{cell::*, coord};

pub fn generate_rank_n(n: u8) -> Cell {
    let mut cell = Cell::new(n);
    if n == 0 {
        return cell;
    }
    let small_cell = generate_rank_n(n - 1);
    let coord_list = vec![
        coord![0, 0],
        coord![0, 1],
        coord![0, 2],
        coord![1, 0],
        coord![1, 1],
        coord![1, 2],
        coord![2, 0],
        coord![2, 1],
        coord![2, 2],
    ];
    for coord in coord_list {
        cell.children.insert(coord, small_cell.clone());
    }
    cell
}
