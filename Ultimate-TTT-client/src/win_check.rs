use crate::board::{Board, Coord};
const DIRECTIONS: [(i16, i16); 4] = [(1, 0), (1, 1), (0, 1), (-1, 1)];

impl Board {
    pub fn is_solved(&self, tiles: &[u8], team_id: u8, pos: Coord) -> bool {
        for dir in DIRECTIONS {
            if count_in_dir(
                pos,
                dir,
                tiles,
                self.layer as i16,
                self.in_a_row as i16,
                team_id,
            ) >= self.in_a_row
            {
                return true;
            }
        }
        false
    }
    pub fn is_tied(&self, tiles: &[u8]) -> bool {
        tiles.iter().all(|x| *x < 255)
    }
}

fn count_in_dir(
    base: Coord,
    dir: (i16, i16),
    tiles: &[u8],
    layer: i16,
    in_a_row: i16,
    team_id: u8,
) -> u8 {
    let mut longest = 0;
    let mut curr = 0;
    for i in (-in_a_row + 1)..in_a_row {
        let offset = offset(base, dir, i);
        if in_bounds(offset, layer) {
            let ind = index(offset, layer);
            curr = if tiles[ind] == team_id { curr + 1 } else { 0 };
            if curr > longest {
                longest = curr;
            }
        }
    }
    longest
}

fn offset(base: Coord, dir: (i16, i16), mag: i16) -> (i16, i16) {
    let offset_x = base.x as i16 + (dir.0 * mag);
    let offset_y = base.y as i16 + (dir.1 * mag);
    (offset_x, offset_y)
}

fn in_bounds(pos: (i16, i16), layer: i16) -> bool {
    0 <= pos.0 && pos.0 < layer && 0 <= pos.1 && pos.1 < layer
}

fn index(pos: (i16, i16), layer: i16) -> usize {
    pos.0 as usize + (layer as usize * pos.1 as usize)
}
