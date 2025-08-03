use crate::TICK_UPDATE_MILLS;
use ratatui::layout::Position;

pub fn human_readable_tick_count(tick_count: u64) -> String {
    let mills = tick_count * TICK_UPDATE_MILLS;
    let seconds = (mills / 1000) % 60;
    let minutes = (mills / (1000 * 60)) % 60;
    let hours = mills / (1000 * 60 * 60);
    format!("playtime: {hours}:{minutes:0>2}:{seconds:0>2}")
}

pub fn xy_to_idx(x: usize, y: usize, grid_width: usize) -> usize {
    (y * grid_width) + x
}

pub fn idx_to_pos(idx: usize, grid_width: usize) -> Position {
    Position {
        x: (idx % grid_width) as u16,
        y: (idx / grid_width) as u16,
    }
}

// SERIOUS BUG: didnt think through case where Position outside of grid
// example: GRID_WIDTH = 250, pos = Position::new(251, 7)
pub fn pos_to_idx(pos: &Position, grid_width: usize) -> usize {
    xy_to_idx(pos.x.into(), pos.y.into(), grid_width)
}

// TODO all should move to checked
pub fn checked_xy_to_idx(x: usize, y: usize, grid_width: usize) -> Option<usize> {
    if y > grid_width || x > grid_width {
        None
    } else {
        Some((y * grid_width) + x)
    }
}

// TODO all should move to checked
pub fn checked_pos_to_idx(pos: &Position, grid_width: usize) -> Option<usize> {
    checked_xy_to_idx(pos.x.into(), pos.y.into(), grid_width)
}

pub fn grid_idx_north(idx: usize, grid_width: usize) -> Option<usize> {
    let pos = idx_to_pos(idx, grid_width);
    if pos.y == 0 {
        None
    } else {
        checked_pos_to_idx(&Position::new(pos.x, pos.y - 1), grid_width)
    }
}

pub fn grid_idx_south(idx: usize, grid_width: usize) -> Option<usize> {
    let pos = idx_to_pos(idx, grid_width);
    checked_pos_to_idx(&Position::new(pos.x, pos.y + 1), grid_width)
}

pub fn grid_idx_east(idx: usize, grid_width: usize) -> Option<usize> {
    let pos = idx_to_pos(idx, grid_width);
    if pos.x == 0 {
        None
    } else {
        checked_pos_to_idx(&Position::new(pos.x - 1, pos.y), grid_width)
    }
}

pub fn grid_idx_west(idx: usize, grid_width: usize) -> Option<usize> {
    let pos = idx_to_pos(idx, grid_width);
    checked_pos_to_idx(&Position::new(pos.x + 1, pos.y), grid_width)
}

pub fn distance(pos1: &Position, pos2: &Position) -> f32 {
    let x1 = pos1.x as f32;
    let y1 = pos1.y as f32;
    let x2 = pos2.x as f32;
    let y2 = pos2.y as f32;
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn there_and_back_again() {
        let pos = Position::new(3, 4);
        let idx = pos_to_idx(&pos, 9);
        let result = idx_to_pos(idx, 9);
        assert_eq!(result, pos);
    }
}
