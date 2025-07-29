use crate::agents::{Agent, UpdateEnum};
use crate::entities::shape::Shape;
use crate::entities::{EntityContainer, PickResult, Properties};
use crate::surface::grid::Grid;
use crate::surface::{GameState, Power};

use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::Color;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use ratatui::buffer::Buffer;
use ratatui::widgets::{Paragraph, WidgetRef};

use std::str::FromStr;

use crate::utils::{idx_to_pos, pos_to_idx, xy_to_idx};

const PLATE_HEIGHT: usize = 12;
const PLATE_WIDTH: usize = 24;

impl WidgetRef for LaserCutter {
    // TODO PERF: store relevant state in LaserCutter so that it is not calculated every render
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let p = Paragraph::new(self.display_text());
        p.render_ref(area, buf);

        let chunks =
            Layout::horizontal([Constraint::Min(PLATE_WIDTH as u16 + 4), Constraint::Fill(1)])
                .split(area);
        let left_area = chunks[0];
        let right_area = chunks[1];

        let right_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(right_area);

        self.buffer_in.render_ref(right_chunks[0], buf);
        self.buffer_out.render_ref(right_chunks[1], buf);

        for (idx, c) in self.plate.iter().enumerate() {
            let unadjusted_pos = idx_to_pos(idx, PLATE_WIDTH);
            //let unadjusted_pos = idx_to_pos_rect(idx, PLATE_HEIGHT, PLATE_WIDTH);
            let pos = Position::new(unadjusted_pos.x + area.x + 3, unadjusted_pos.y + area.y + 3);
            if left_area.contains(pos) {
                if *c == State::Uncut {
                    if let Some(cell) = buf.cell_mut(pos) {
                        cell.set_fg(Color::Black);
                        cell.set_bg(Color::Green);
                    }
                } else if let Some(cell) = buf.cell_mut(pos) {
                    cell.set_bg(Color::Black);
                    if self.laser_is_on && unadjusted_pos == self.laser_position() {
                        cell.set_fg(Color::Red);
                    } else {
                        cell.set_fg(Color::Green);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum State {
    Cut,
    Uncut,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaserCutter {
    pub x_servo: usize,
    pub y_servo: usize,

    pub plate: Vec<State>,
    pub plate_kind: Option<Properties>,

    pub laser_is_on: bool,

    pub integrity: u8,
    pub buffer_in: EntityContainer,
    pub buffer_out: EntityContainer,
}

#[typetag::serde]
impl Agent for LaserCutter {
    fn new() -> Self {
        Self {
            x_servo: 0,
            y_servo: 0,
            //plate: vec![State::Uncut; PLATE_SIZE.pow(2)],
            plate: vec![State::Cut; PLATE_WIDTH * PLATE_HEIGHT],
            plate_kind: None,
            laser_is_on: false,
            integrity: 100,
            buffer_in: EntityContainer::new("BUFFER IN", 10),
            buffer_out: EntityContainer::new("BUFFER OUT", 10),
        }
    }

    fn handle_message(
        &mut self,
        _pos: &Position,
        _grid: &mut Grid,
        _game_state: &mut GameState,
        msg: String,
    ) -> UpdateEnum {
        match Self::parse_command(&msg) {
            Ok(command) => self.handle_command(command),
            Err(e) => UpdateEnum::reply(format!("ERRR: {e}")),
        }
    }

    fn tick(&mut self, _power: &mut Power) {}

    fn properties(&self) -> Properties {
        Properties::LaserCutter
    }
    fn pick(&mut self, c: char) -> PickResult {
        let pick_result = self.buffer_out.pick(c);
        if pick_result.picked.is_some() {
            pick_result
        } else {
            self.buffer_in.pick(c)
        }
    }

    fn placable(&self, _prop: &Properties) -> bool {
        self.buffer_in.placable()
    }
    fn place(&mut self, prop: Properties) {
        self.buffer_in.place(prop);
    }
}

fn tens_digit_char(i: usize) -> char {
    let tens_digit = (i as u32 + 1) / 10;
    if tens_digit == 0 {
        ' '
    } else {
        char::from_digit(tens_digit, 10).expect("less than 10")
    }
}

fn singles_digit_char(i: usize) -> char {
    let tens_digit = (i as u32 + 1) % 10;
    char::from_digit(tens_digit, 10).expect("less than 10")
}

impl LaserCutter {
    fn laser_position(&self) -> Position {
        Position::new(self.x_servo as u16, self.y_servo as u16)
    }

    fn display_text(&self) -> String {
        let mut plate = String::new();

        // first row: tens digits
        plate.push(' ');
        plate.push(' ');
        plate.push(' ');
        for x in 0..PLATE_WIDTH {
            plate.push(tens_digit_char(x));
        }
        plate.push('\n');

        // second row: tens digits
        plate.push(' ');
        plate.push(' ');
        plate.push(' ');
        for x in 0..PLATE_WIDTH {
            plate.push(singles_digit_char(x));
        }
        plate.push('\n');

        // third row: frame
        plate.push(' ');
        plate.push(' ');
        plate.push('┌');
        for i in 0..PLATE_WIDTH {
            if i == self.x_servo {
                plate.push('┰');
            } else {
                plate.push('─');
            }
        }
        plate.push('\n');
        for (y, row) in self.plate.chunks(PLATE_WIDTH).enumerate() {
            plate.push(tens_digit_char(y));
            plate.push(singles_digit_char(y));
            if y == self.y_servo {
                plate.push('┝');
            } else {
                plate.push('│');
            }
            for (x, _todo) in row.iter().enumerate() {
                match (x == self.x_servo, y == self.y_servo, self.laser_is_on) {
                    (true, true, true) => plate.push('◊'),
                    (true, true, false) => plate.push('┼'),
                    (true, false, _) => plate.push('╎'),
                    (false, true, _) => plate.push('╌'),
                    (false, false, _) => plate.push(' '),
                }
            }
            plate.push('\n');
        }

        plate
    }

    fn laser_idx(&self) -> usize {
        xy_to_idx(self.x_servo, self.y_servo, PLATE_WIDTH)
    }

    fn update_plate(&mut self) {
        if self.laser_is_on {
            let idx = self.laser_idx();
            self.plate[idx] = State::Cut;
        }
    }

    fn handle_command(&mut self, command: Command) -> UpdateEnum {
        match command {
            Command::POWR => {
                self.laser_is_on = !self.laser_is_on;
                self.update_plate();
                UpdateEnum::okay()
            }
            Command::PICK(p) => {
                let idx = pos_to_idx(
                    &Position::new(self.x_servo as u16, self.y_servo as u16),
                    PLATE_WIDTH,
                );
                if self.plate[idx] == State::Cut {
                    return UpdateEnum::reply("ERRR no material to pick");
                }
                tracing::info!("p: {p:?}");
                let (expected_material, expected_shape) = p
                    .material_and_shape()
                    .expect("command only accepts properties with a shape");
                if Some(expected_material) != self.plate_kind {
                    return UpdateEnum::reply("ERRR incorrect material");
                }
                let found_shape = Solver::new(
                    self.plate.clone(),
                    Position::new(self.x_servo as u16, self.y_servo as u16),
                )
                .solve();
                tracing::info!("expected_shape:\n{expected_shape:?}");
                tracing::info!("found_shape:\n{found_shape:?}");
                tracing::info!(
                    "found_shape.normalize:\n{:?}",
                    found_shape.clone().normalize()
                );
                tracing::info!("expected_shape:\n{expected_shape}");
                tracing::info!("found_shape:\n{found_shape}");
                tracing::info!(
                    "found_shape.normalize:\n{}",
                    found_shape.clone().normalize()
                );
                if expected_shape == found_shape.clone().normalize() {
                    self.buffer_out.place(p);
                    for pos in found_shape.positions {
                        self.plate[pos_to_idx(&pos, PLATE_WIDTH)] = State::Cut;
                    }
                    UpdateEnum::okay()
                } else {
                    UpdateEnum::reply("ERRR shape does not match")
                }
            }
            Command::LOAD(prop) => {
                if self.laser_is_on {
                    return UpdateEnum::reply("ERRR laser is on");
                }
                // TODO match prop
                // TODO bug here: need to have method on buffer_in to remove entity
                // otherwise we forget to update buffer_in display
                match self.buffer_in.remove_entity(&prop) {
                    Ok(_) => {
                        // TODO implement scrap system
                        //let amount = self.plate.iter().filter(|x| **x == State::Uncut).count();
                        //self.buffer_out.place(Properties::Scrap(amount));
                        self.plate = vec![State::Uncut; PLATE_WIDTH * PLATE_HEIGHT];
                        self.plate_kind = Some(prop);
                        UpdateEnum::okay()
                    }
                    Err(_) => UpdateEnum::reply(format!("ERRR {prop}")),
                }
            }
            Command::MVXP => {
                self.x_servo = self.x_servo.saturating_add(1);
                self.update_plate();
                UpdateEnum::okay()
            }
            Command::MVXN => {
                self.x_servo = self.x_servo.saturating_sub(1);
                self.update_plate();
                UpdateEnum::okay()
            }
            Command::MVYP => {
                self.y_servo = self.y_servo.saturating_add(1);
                self.update_plate();
                UpdateEnum::okay()
            }
            Command::MVYN => {
                self.y_servo = self.y_servo.saturating_sub(1);
                self.update_plate();
                UpdateEnum::okay()
            }
            Command::STAT => UpdateEnum::reply(Reply::STAT {
                buffer_in: self
                    .buffer_in
                    .content
                    .iter()
                    .map(|p| p.character())
                    .collect(),
                buffer_out: self
                    .buffer_out
                    .content
                    .iter()
                    .map(|p| p.character())
                    .collect(),
            }),
        }
    }

    fn parse_command(msg: &str) -> Result<Command, String> {
        match msg {
            "STAT" => Ok(Command::STAT),
            "POWR" => Ok(Command::POWR),
            "MVXP" => Ok(Command::MVXP),
            "MVXN" => Ok(Command::MVXN),
            "MVYP" => Ok(Command::MVYP),
            "MVYN" => Ok(Command::MVYN),
            x if x.starts_with("LOAD") => {
                let kind = x.split_whitespace().nth(1).unwrap_or_default();
                if let Some(prop) = Properties::from_user_input(kind)
                {
                    if prop.cuttable() {
                        Ok(Command::LOAD(prop))
                    } else {
                        Err(format!("cannot cut {kind}"))
                    }
                } else {
                    Err(format!("unknown entity {kind}"))
                }
            },
            x if x.starts_with("PICK") => {
                let kind = x.split_whitespace().nth(1).unwrap_or_default();
                if let Some(prop) = Properties::from_user_input(kind)
                {
                    if let Some(_) = prop.material_and_shape() {
                        Ok(Command::PICK(prop))
                    } else {
                        Err(format!("{kind} is not made in laser cutter"))
                    }
                } else {
                    Err(format!("unknown entity {kind}"))
                }
            }
            _ => Err(format!("unknow command: {msg}")),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
enum Command {
    POWR,
    PICK(Properties),
    LOAD(Properties),
    MVXP,
    MVXN,
    MVYP,
    MVYN,
    STAT,
}

pub const DOCUMENTATION: &str = "LASER CUTTER

  cut the correct shapes from IRON_PLATEs and COPPER_PLATEs to create new entities

  for the required shapes see Documentation > Entities 

   entity      | cut from
  -------------+--------------
   GEAR        | IRON_PLATE
   NUT         | IRON_PLATE
   BAR_WINDING | COPPER_PLATE

COMMANDS
  
  POWR
    toggle the laser on/off, consuming 100j 
    when the laser is on, plate at the laser position will be cut

  MVXP
    move the laser one position right

  MVXN
    move the laser one position left

  MVYP
    move the laser one position down

  MVYN
    move the laser one position up

  PICK intermediate={GEAR|NUT|BAR_WINDING|...}
    checks the location of the laser for the intermediate's shape
    if the correct shape is found, the shape is removed and the intermediate 
    is added to the output buffer

    Usage:
      PICK GEAR  ->  OKAY   #GEAR added to output buffer
      PICK GEAR  ->  ERRR shape does not match

  LOAD plate={COPPER_PLATE|IRON_PLATE|WAFER}
    move material from the input buffer into the laser cutter
    uncut material in the laser cutter will be scrapped
      
  STAT
    returns the content of the input and output buffers

    Usage:
      STAT  ->  STAT _ GG    #the output buffer contains two GEARs
      STAT  ->  STAT P _     #the input buffer contains one PLATE
";

#[derive(Debug)]
pub enum Reply {
    ERRR(String),
    STAT {
        buffer_in: Vec<char>,
        buffer_out: Vec<char>,
    },
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reply::ERRR(msg) => write!(f, "ERRR {msg}"),
            Reply::STAT {
                buffer_in,
                buffer_out,
            } => {
                write!(f, "STAT ")?;
                if buffer_in.is_empty() {
                    write!(f, "_")?;
                } else {
                    for c in buffer_in {
                        write!(f, "{c}")?;
                    }
                }
                write!(f, " ")?;
                if buffer_out.is_empty() {
                    write!(f, "_")?;
                } else {
                    for c in buffer_out {
                        write!(f, "{c}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

struct Solver {
    plate: Vec<State>,
    // TODO find better name
    unexhausted: HashSet<Position>,
    // TODO find better name
    exhausted: HashSet<Position>,
}

impl Solver {
    fn new(plate: Vec<State>, pos: Position) -> Solver {
        let idx = pos_to_idx(&pos, PLATE_WIDTH);
        if plate[idx] == State::Uncut {
            tracing::error!("expected to be uncut!");
        };
        Solver {
            plate,
            unexhausted: HashSet::from([pos]),
            exhausted: HashSet::new(),
        }
    }

    /// assumes self.plate[pos] == State::Cut
    fn get_uncut_neighbors(&mut self, pos: Position) -> Vec<Position> {
        self.exhausted.insert(pos);
        let mut uncut_neighbors = Vec::new();
        let x_adjust = vec![-1, 0, 1];
        let y_adjust = vec![-1, 0, 1];
        for x in x_adjust.into_iter() {
            for y in y_adjust.clone().into_iter() {
                let x = (i32::from(pos.x) + x).unsigned_abs() as usize;
                let y = (i32::from(pos.y) + y).unsigned_abs() as usize;
                let idx = xy_to_idx(x, y, PLATE_WIDTH);
                if let Some(s) = self.plate.get_mut(idx) {
                    if *s == State::Uncut {
                        *s = State::Cut;
                        uncut_neighbors.push(idx_to_pos(idx, PLATE_WIDTH));
                    }
                }
            }
        }
        uncut_neighbors
    }

    fn solve(mut self) -> Shape {
        while !self.unexhausted.is_empty() {
            //let mut new_unexhausted = HashSet::new();
            let positions: Vec<Position> = self.unexhausted.drain().collect();
            for pos in positions {
                for uncut_neighbor in self.get_uncut_neighbors(pos) {
                    self.unexhausted.insert(uncut_neighbor);
                }
            }
        }
        Shape {
            positions: self.exhausted,
        }
    }
}

// TODO make *_to_* and *_from_* functions proper From<> implementations
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::shape::Shape;

    fn plate_from_string(s: String) -> Vec<State> {
        s.chars()
            .filter(|c| *c != '\n')
            .map(|c| if c == '.' { State::Cut } else { State::Uncut })
            .collect()
    }

    #[test]
    fn test_case1() {
        let input = "
.........
.xxxxxx..
.xx...x..
......x..
.........
.........
.........
.........
........."
            .trim()
            .to_string();
        let plate = plate_from_string(input.clone());
        println!("plate: {plate:?}");

        let solver = Solver::new(plate, Position::new(1, 1));
        let result = solver.solve();

        let expected = Shape::from(&input);

        println!("expected:");
        println!("{expected}");
        println!("result:");
        println!("{result}");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_case2() {
        let input = "
.........
.xxxxxx..
.xx...x.x
......x.x
..xxx....
....x....
....xx...
.x.......
........."
            .trim()
            .to_string();
        let expected = "
.........
.xxxxxx..
.xx...x..
......x..
.........
.........
.........
.........
........."
            .trim()
            .to_string();
        let expected = Shape::from(&expected);
        let plate = plate_from_string(input.clone());
        println!("plate: {plate:?}");

        let solver = Solver::new(plate, Position::new(1, 1));
        let result = solver.solve();

        println!("expected:");
        println!("{expected}");
        println!("result:");
        println!("{result}");

        assert_eq!(result, expected);
    }

    #[test]
    fn test_case3() {
        let input = "
xx.......
x........
.........
.........
.........
.........
.........
.........
........."
            .trim()
            .to_string();
        let plate = plate_from_string(input.clone());
        println!("plate: {plate:?}");

        let solver = Solver::new(plate, Position::new(0, 0));
        let result = solver.solve();

        let expected = Shape::from(&input);

        println!("expected:");
        println!("{expected}");
        println!("result:");
        println!("{result}");

        assert_eq!(result, expected);
    }
}
