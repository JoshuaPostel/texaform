use rand::Rng;
use rand_chacha::ChaCha8Rng;
use ratatui::layout::Position;
use std::collections::{BTreeSet, HashSet};

use crate::utils::{
    checked_pos_to_idx, distance, grid_idx_east, grid_idx_north, grid_idx_south, grid_idx_west,
};

pub const GEAR: &str = "
......OO......
.OO...OO...OO.
..OOOOOOOOOO..
...OO....OO...
OOOO......OOOO
...OO....OO...
..OOOOOOOOOO..
.OO...OO...OO.
......OO......
";

pub const NUT: &str = "
...xxxxxxxx...
..xxxxxxxxxx..
.xxxx....xxxx.
xxxx......xxxx
.xxxx....xxxx.
..xxxxxxxxxx..
...xxxxxxxx...
";

pub const BAR_WINDING: &str = "
......O......
....OOOOO....
..OOO...OOO..
OOO.......OOO
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
O...........O
";

pub const STATOR: &str = "
....OOOOOOOOO....
..OO  OOOOO  OO..
.OOOO  OOO  OOOO.
.OOOOO     OOOOO.
OO             OO
.OOOOO     OOOOO.
.OOOO  OOO  OOOO.
..OO  OOOOO  OO..
....OOOOOOOOO....
";

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub positions: HashSet<Position>,
}

impl Shape {
    pub fn from(s: &str) -> Shape {
        let s = s.trim();
        let mut positions = HashSet::new();
        let mut x = 0;
        let mut y = 0;
        for c in s.chars() {
            if c == '\n' {
                x = 0;
                y += 1;
            } else if c != '.' {
                positions.insert(Position::new(x, y));
                x += 1;
            } else {
                x += 1;
            }
        }
        Shape { positions }
    }

    pub fn normalize(self) -> Shape {
        let x_min = self.positions.iter().map(|p| p.x).min().unwrap_or(0);
        let y_min = self.positions.iter().map(|p| p.y).min().unwrap_or(0);
        let positions = self
            .positions
            .iter()
            .map(|p| Position::new(p.x - x_min, p.y - y_min))
            .collect();
        Shape { positions }
    }

    pub fn circle(radius: u16) -> Shape {
        let mut positions = HashSet::new();
        let center = Position::new(radius, radius);
        positions.insert(center);
        for x in 0..(radius * 2) + 1 {
            for y in 0..(radius * 2) + 1 {
                let pos = Position::new(x, y);
                let dist = distance(&center, &pos);
                let include = if radius < 4 {
                    dist <= radius as f32
                } else {
                    dist < radius as f32
                };
                if include {
                    positions.insert(pos);
                }
                // looks better when we dont include == radius points
                // if distance(&center, &pos) <= radius as f32 {
                // if distance(&center, &pos) < radius as f32 {
                //      positions.insert(pos);
                // }
            }
        }
        let shape = Shape { positions };
        shape.normalize()
    }

    fn is_edge(&self, pos: &Position) -> bool {
        if pos.x == 0 || pos.y == 0 {
            true
        } else {
            !self.positions.contains(&Position::new(pos.x, pos.y - 1))
                || !self.positions.contains(&Position::new(pos.x, pos.y + 1))
                || !self.positions.contains(&Position::new(pos.x - 1, pos.y))
                || !self.positions.contains(&Position::new(pos.x + 1, pos.y))
        }
    }

    // TODO PERF: cache this
    // need BTreeSet for rand determinisim
    pub fn edge(&self) -> BTreeSet<Position> {
        let mut edge = BTreeSet::new();
        for pos in &self.positions {
            if self.is_edge(pos) {
                edge.insert(*pos);
            }
        }
        edge
    }

    // TODO better name than footprint
    pub fn grid_footprint(&self, grid_pos: &Position, grid_width: usize) -> HashSet<usize> {
        let mut footprint = HashSet::new();
        for offset in &self.positions {
            let adj_pos = Position::new(grid_pos.x + offset.x, grid_pos.y + offset.y);
            if let Some(grid_idx) = checked_pos_to_idx(&adj_pos, grid_width) {
                footprint.insert(grid_idx);
                if self.is_edge(offset) {
                    if let Some(idx) = grid_idx_north(grid_idx, grid_width) {
                        footprint.insert(idx);
                    }
                    if let Some(idx) = grid_idx_south(grid_idx, grid_width) {
                        footprint.insert(idx);
                    }
                    if let Some(idx) = grid_idx_east(grid_idx, grid_width) {
                        footprint.insert(idx);
                    }
                    if let Some(idx) = grid_idx_west(grid_idx, grid_width) {
                        footprint.insert(idx);
                    }
                }
            }
        }
        footprint
    }

    pub fn jitter_edge(&mut self, rng: &mut ChaCha8Rng) {
        let edge = self.edge();
        for pos in edge {
            if rng.random() {
                self.positions.remove(&pos);
            }
        }
    }

    pub fn jittered_circle(rng: &mut ChaCha8Rng, radius: u16, iters: u16) -> Shape {
        let mut shape = Shape::circle(radius);
        for _ in 0..iters + 1 {
            shape.jitter_edge(rng);
        }
        shape
    }

    pub fn wave(rng: &mut ChaCha8Rng, length: u16, bend_chance: f32, horizontal: bool) -> Shape {
        let mut positions = HashSet::new();
        if horizontal {
            let mut y = length;
            for x in length..length * 2 {
                positions.insert(Position::new(x, y));
                if rng.random::<f32>() < bend_chance {
                    if rng.random() {
                        y += 1;
                    } else {
                        y = y.saturating_sub(1);
                    }
                }
            }
        } else {
            let mut x = length;
            for y in length..length * 2 {
                positions.insert(Position::new(x, y));
                if rng.random::<f32>() < bend_chance {
                    if rng.random::<bool>() {
                        x += 1;
                    } else {
                        x = x.saturating_sub(1);
                    }
                }
            }
        }
        let shape = Shape { positions };
        shape.normalize()
    }

    pub fn waffle_fry(
        rng: &mut ChaCha8Rng,
        length: u16,
        bend_chance: f32,
        horizontal: bool,
    ) -> Shape {
        let (w1, w2, w3) = if horizontal {
            (
                Shape::wave(rng, length, bend_chance, horizontal).translate(1, 0),
                Shape::wave(rng, length + 2, bend_chance, horizontal).translate(0, 3),
                Shape::wave(rng, length, bend_chance, horizontal).translate(1, 6),
            )
        } else {
            (
                Shape::wave(rng, length, bend_chance, horizontal).translate(0, 1),
                Shape::wave(rng, length + 2, bend_chance, horizontal).translate(3, 0),
                Shape::wave(rng, length, bend_chance, horizontal).translate(6, 1),
            )
        };
        let mut positions = HashSet::new();
        positions.extend(w1.positions);
        positions.extend(w2.positions);
        positions.extend(w3.positions);
        Shape { positions }
    }

    pub fn diamond() -> Shape {
        let positions = HashSet::from([
            Position::new(1, 0),
            Position::new(0, 1),
            Position::new(2, 1),
            Position::new(1, 2),
        ]);
        Shape { positions }
    }

    pub fn translate(self, x: u16, y: u16) -> Shape {
        let mut positions = HashSet::new();
        for pos in self.positions {
            positions.insert(Position::new(pos.x + x, pos.y + y));
        }
        Shape { positions }
    }

    // TODO matrix transforms for rotation
    //
    // TODO transform for translation
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let x_max = self.positions.iter().map(|p| p.x).max().unwrap_or(0);
        let y_max = self.positions.iter().map(|p| p.y).max().unwrap_or(0);
        let mut s = String::new();
        for y in 0..y_max + 1 {
            for x in 0..x_max + 1 {
                if self.positions.contains(&Position::new(x, y)) {
                    s.push('X');
                } else {
                    s.push('.');
                }
            }
            s.push('\n');
        }
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_shape() {
        let input = "
xx..
x...
....
....
";

        let expected = "XX
X.
";
        let result = Shape::from(input).to_string();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_shape() {
        let input = "
xx..
x...
....
....
";
        let positions = [
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(0, 1),
        ];
        let positions = HashSet::from(positions);
        let expected = Shape { positions };
        let result = Shape::from(input);

        assert_eq!(expected, result);
    }

    #[test]
    fn test_normalize() {
        let normal = "
xx..
x...
....
....
";

        let shifted = "
....
....
..xx
..x.
";
        let expected = Shape::from(normal);
        let result = Shape::from(shifted).normalize();

        assert_eq!(expected, result);
    }

    #[test]
    fn test_shapes() {
        let expected = "XX
X.
";
        let mut result = Shape::wave(12, 0.2, true);
        println!("{result}");
        let result = result.translate(2, 0);
        println!("{result}");
        let result = result.translate(2, 0);
        println!("{result}");
        //        let edge = Shape { positions: result.clone().edge() };
        //        println!("{edge}");
        //        result.jitter_edge();
        //        println!("{result}");
        //        result.jitter_edge();
        //        println!("{result}");
        //        result.jitter_edge();
        //        println!("{result}");
        //        result.jitter_edge();
        //        println!("{result}");
        //        result.jitter_edge();
        //        println!("{result}");

        assert_eq!(expected, result.to_string());
    }

    //    use crate::entities::properties::Properties;
    //
    //    #[test]
    //    fn test_shape_from2() {
    //        let input = "
    //xx..
    //x...
    //....
    //....
    //";
    //        let positions = [Position::new(0, 0), Position::new(1, 0), Position::new(0, 1)];
    //        let positions = HashSet::from(positions);
    //        let expected = Shape { positions };
    //        let result = Shape::from(input);
    //        let result = Shape::from(GEAR);
    //        let result = Shape::from(NUT);
    //        let p = Properties::Nut;
    //        let result = p.shape().expect("");
    //
    //        assert_eq!(expected, result);
    //    }
    //
}
