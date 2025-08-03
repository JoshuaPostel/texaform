use crate::agents::{Agent, UpdateEnum};
use crate::entities::{EntityContainer, Properties};
use crate::surface::grid::{Gent, Grid};
use crate::surface::{GameState, Power};
use crate::ui::render_effect_clamped;

use ratatui::Frame;
use ratatui::buffer::Cell;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Style};

use tracing::*;

use serde::{Deserialize, Serialize};

use tachyonfx::{Effect, Shader, fx};

use ratatui::buffer::Buffer;
use ratatui::widgets::{Gauge, WidgetRef};

const MAX_BATTERY: usize = 10000;

impl WidgetRef for Dog {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical([Constraint::Max(1), Constraint::Max(3)]).split(area);
        let battery_percent = (self.battery as f64 / MAX_BATTERY as f64) * 100.0;

        Gauge::default()
            .gauge_style(Style::new().fg(Color::Green).bg(Color::Black))
            .label(format!("Battery: {battery_percent:.2}%"))
            .percent(battery_percent as u16)
            .render_ref(rows[0], buf);

        self.payload.render_ref(rows[1], buf);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dog {
    facing: CardinalDirection,
    payload: EntityContainer,
    battery: usize,
    charging: bool,
    // TODO we might want to do the "repository approach here too"
    #[serde(skip)]
    effects: Vec<Effect>,
}

impl std::fmt::Debug for Dog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "foo")
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CardinalDirection {
    North,
    South,
    East,
    West,
}

impl std::fmt::Display for CardinalDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CardinalDirection::North => write!(f, "N"),
            CardinalDirection::South => write!(f, "S"),
            CardinalDirection::East => write!(f, "E"),
            CardinalDirection::West => write!(f, "W"),
        }
    }
}

impl CardinalDirection {
    fn left(&self) -> Self {
        match self {
            CardinalDirection::North => CardinalDirection::West,
            CardinalDirection::West => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::East,
            CardinalDirection::East => CardinalDirection::North,
        }
    }

    fn right(&self) -> Self {
        match self {
            CardinalDirection::North => CardinalDirection::East,
            CardinalDirection::East => CardinalDirection::South,
            CardinalDirection::South => CardinalDirection::West,
            CardinalDirection::West => CardinalDirection::North,
        }
    }
}

#[typetag::serde]
impl Agent for Dog {
    fn new() -> Dog {
        Dog {
            facing: CardinalDirection::East,
            payload: EntityContainer::new("Payload", 1),
            battery: MAX_BATTERY,
            charging: false,
            effects: vec![],
        }
    }

    fn handle_message(
        &mut self,
        pos: &Position,
        grid: &mut Grid,
        _game_state: &mut GameState,
        msg: String,
    ) -> UpdateEnum {
        match Self::parse_command(&msg) {
            Ok(command) => self.handle_command(pos, grid, command),
            Err(e) => UpdateEnum::reply(format!("ERRR: {e}")),
        }
    }

    fn tick(&mut self, power: &mut Power) {
        if self.charging {
            if power.consume(500) {
                self.battery = (self.battery + 500).min(MAX_BATTERY);
            }
        } else {
            // charging due to solar power
            self.battery = MAX_BATTERY.min(self.battery + 1);
        }
    }
    fn properties(&self) -> Properties {
        Properties::Dog
    }
    fn render_fx(
        &mut self,
        grid_pos: &Position,
        frame: &mut Frame,
        area: Rect,
        prev_tick: core::time::Duration,
    ) {
        // TODO called too often? move to tick?
        self.effects.retain(|e| e.running());
        let x = grid_pos.x + area.x;
        let y = grid_pos.y + area.y;
        let scan_area = match self.facing {
            // TODO if at border x and y get set to zero, and width/height of 3 is no longer
            // correct
            CardinalDirection::North => Rect {
                x,
                y: y.saturating_sub(3),
                width: 1,
                height: 3,
            },
            CardinalDirection::South => Rect {
                x,
                y: y.saturating_add(1),
                width: 1,
                height: 3,
            },
            CardinalDirection::East => Rect {
                x: x.saturating_add(1),
                y,
                width: 3,
                height: 1,
            },
            CardinalDirection::West => Rect {
                x: x.saturating_sub(3),
                y,
                width: 3,
                height: 1,
            },
        };
        for effect in self.effects.iter_mut() {
            render_effect_clamped(frame, effect, scan_area, prev_tick)
        }
    }
    fn render_surface_cell(&self, _offset: &Position, cell: &mut Cell) {
        //        let connection_color = match self.comms.address {
        //            Some(_) => Color::Green,
        //            None => Color::Red,
        //        };
        //        cell.bg = connection_color;
        // TODO
        cell.bg = Color::Yellow;
        cell.fg = Color::White;
        cell.set_char(self.get_char());
    }
    fn place(&mut self, prop: Properties) {
        self.payload.place(prop);
    }

    fn placable(&self, _prop: &Properties) -> bool {
        self.payload.placable()
    }
}

impl Dog {
    fn handle_command(&mut self, pos: &Position, grid: &mut Grid, command: Command) -> UpdateEnum {
        //        info!(
        //            "update start: x: {}, y: {}, facing: {:?}, payload: {:?}",
        //            pos.x, pos.y, self.facing, self.payload
        //        );

        if self.charging {
            self.charging = false;
        }
        let energy = command.energy_cost();
        if self.battery < energy {
            self.battery = 0;
            return UpdateEnum::reply(Reply::ERRR("low battery".to_string()));
        }
        self.battery = self.battery.saturating_sub(energy);
        match command {
            Command::TURN(Direction::L) => {
                self.facing = self.facing.left();
                UpdateEnum::okay()
            }
            Command::TURN(Direction::R) => {
                self.facing = self.facing.right();
                UpdateEnum::okay()
            }
            Command::STAT => UpdateEnum::reply(Reply::STAT {
                facing: self.facing,
                position: *pos,
                battery: self.battery,
                payload: self.payload.content_chars.iter().collect(),
            }),
            Command::MOVE => {
                let forward = self.cordinites_forward(pos);
                match grid.get_mut(&forward) {
                    Some(Gent::Empty) => UpdateEnum::Move(forward),
                    Some(_gent) => UpdateEnum::reply(Reply::ERRR("crashed".to_string())),
                    None => UpdateEnum::reply(Reply::ERRR("out of bounds".to_string())),
                }
            }
            Command::PICK(prop) => {
                if !self.payload.placable() {
                    UpdateEnum::reply(Reply::ERRR("already full".to_string()))
                } else {
                    let forward = self.cordinites_forward(pos);
                    match grid.pick(prop.character(), &forward) {
                        Some(gent) => {
                            self.payload.place(gent);
                            UpdateEnum::okay()
                        }
                        None => UpdateEnum::reply(Reply::ERRR(format!("no {prop} to pick"))),
                    }
                }
            }
            Command::CHRG => {
                let forward = self.cordinites_forward(pos);
                if Some(Properties::Accumulator) == grid.get(&forward).map(|gent| gent.properties())
                {
                    self.charging = true;
                    UpdateEnum::okay()
                } else {
                    UpdateEnum::reply(Reply::ERRR("not facing an accumulator".to_string()))
                }
            }
            // TODO DESIGN: do we want ability to drop specific item?
            Command::DROP => {
                let forward = self.cordinites_forward(pos);
                if let Some(first_prop) = self.payload.pop() {
                    match grid.get_mut(&forward) {
                        Some(Gent::Empty) => {
                            grid.insert(&forward, Gent::Intmd(first_prop));
                            UpdateEnum::okay()
                        }
                        Some(Gent::Age(entity)) => {
                            if entity.placable(&first_prop) {
                                entity.place(first_prop);
                                UpdateEnum::okay()
                            } else {
                                self.payload.place(first_prop);
                                UpdateEnum::reply(Reply::ERRR("cannot drop here".to_string()))
                            }
                        }
                        Some(_) => {
                            self.payload.place(first_prop);
                            UpdateEnum::reply(Reply::ERRR("cannot drop here".to_string()))
                        }
                        None => {
                            self.payload.place(first_prop);
                            UpdateEnum::reply(Reply::ERRR("out of bounds".to_string()))
                        }
                    }
                } else {
                    UpdateEnum::reply(Reply::ERRR("payload is empty".to_string()))
                }
            }
            Command::BULD => {
                if let Some(prop) = self.payload.pop() {
                    if let Some(fp) = prop.footprint() {
                        if let (Some(x), Some(y)) = match self.facing {
                            CardinalDirection::North => (Some(pos.x), pos.y.checked_sub(fp.y)),
                            CardinalDirection::South => {
                                (pos.x.checked_sub(fp.x - 1), pos.y.checked_add(1))
                            }
                            CardinalDirection::East => (pos.x.checked_add(1), Some(pos.y)),
                            CardinalDirection::West => {
                                (pos.x.checked_sub(fp.x), pos.y.checked_sub(fp.y - 1))
                            }
                        } && grid.buildable(Rect {
                            x,
                            y,
                            width: fp.x,
                            height: fp.y,
                        }) {
                            if let Ok(agent) = prop.create_agent() {
                                UpdateEnum::BuildAgent {
                                    pos: Position::new(x, y),
                                    agent,
                                }
                            } else {
                                UpdateEnum::BuildEntity {
                                    pos: Position::new(x, y),
                                    entity: prop,
                                }
                            }
                        } else {
                            UpdateEnum::reply(Reply::ERRR(format!("location not buildable")))
                        }
                    } else {
                        UpdateEnum::reply(Reply::ERRR(format!("{prop} is not buildable")))
                    }
                } else {
                    UpdateEnum::reply(Reply::ERRR("no payload to build".to_string()))
                }
            }
            // 3 forward scan
            Command::SCAN => {
                //self.effects.retain(|e| e.running());
                self.effects
                    .push(fx::fade_from(Color::White, Color::Green, 20000));
                // using very long animation rn for testing
                //self.effects.push(fx::fade_from(Color::White, Color::Green, 2000));
                tracing::info!("effecs: {}", self.effects.len());
                let (p1, p2, p3) = match self.facing {
                    CardinalDirection::East => {
                        let p1 = Position {
                            x: pos.x.saturating_add(1),
                            y: pos.y,
                        };
                        let p2 = Position {
                            x: pos.x.saturating_add(2),
                            y: pos.y,
                        };
                        let p3 = Position {
                            x: pos.x.saturating_add(3),
                            y: pos.y,
                        };
                        (p1, p2, p3)
                    }
                    CardinalDirection::West => {
                        let p1 = Position {
                            x: pos.x.saturating_sub(1),
                            y: pos.y,
                        };
                        let p2 = Position {
                            x: pos.x.saturating_sub(2),
                            y: pos.y,
                        };
                        let p3 = Position {
                            x: pos.x.saturating_sub(3),
                            y: pos.y,
                        };
                        (p1, p2, p3)
                    }
                    CardinalDirection::South => {
                        let p1 = Position {
                            x: pos.x,
                            y: pos.y.saturating_add(1),
                        };
                        let p2 = Position {
                            x: pos.x,
                            y: pos.y.saturating_add(2),
                        };
                        let p3 = Position {
                            x: pos.x,
                            y: pos.y.saturating_add(3),
                        };
                        (p1, p2, p3)
                    }
                    CardinalDirection::North => {
                        let p1 = Position {
                            x: pos.x,
                            y: pos.y.saturating_sub(1),
                        };
                        let p2 = Position {
                            x: pos.x,
                            y: pos.y.saturating_sub(2),
                        };
                        let p3 = Position {
                            x: pos.x,
                            y: pos.y.saturating_sub(3),
                        };
                        (p1, p2, p3)
                    }
                };
                let c1 = grid
                    .get(&p1)
                    .map(|g| g.properties().character())
                    .unwrap_or('.');
                let c2 = grid
                    .get(&p2)
                    .map(|g| g.properties().character())
                    .unwrap_or('.');
                let c3 = grid
                    .get(&p3)
                    .map(|g| g.properties().character())
                    .unwrap_or('.');
                let mut area = String::new();
                area.push(c1);
                area.push(c2);
                area.push(c3);

                UpdateEnum::reply(Reply::AREA(area))
            } // 3 by 3 scan
              // Command::SCAN => {
              //     let rect = match self.facing {
              //         CardinalDirection::East => Rect {
              //             x: pos.x.saturating_add(1),
              //             y: pos.y.saturating_sub(1),
              //             width: 3,
              //             height: 3,
              //         },
              //         _ => todo!(),
              //     };
              //     let area: String = rect
              //         .positions()
              //         .map(|pos| surface.grid.get(&pos).map(|g| g.character()).unwrap_or('.'))
              //         .collect();
              //     UpdateEnum::reply(Reply::AREA(area))
              // }
              // one forware scan
              // Command::SCAN => {
              //     let forward = self.cordinites_forward(pos);
              //     match surface.grid.get(&forward) {
              //         Some(gent) => UpdateEnum::reply(Reply::AREA(gent.character().to_string())),
              //         None => UpdateEnum::reply(Reply::ERRR("out of bounds".to_string())),
              //     }
              // }
        }
    }

    fn get_char(&self) -> char {
        match self.facing {
            CardinalDirection::North => 'ʌ',
            CardinalDirection::South => 'v',
            CardinalDirection::East => '>',
            CardinalDirection::West => '<',
        }
    }

    /// TODO returns current cordinates if forward is off the map
    fn cordinites_forward(&self, pos: &Position) -> Position {
        match self.facing {
            CardinalDirection::North => Position {
                x: pos.x,
                y: pos.y.saturating_sub(1),
            },
            CardinalDirection::South => Position {
                x: pos.x,
                y: pos.y.saturating_add(1),
            },
            CardinalDirection::East => Position {
                x: pos.x.saturating_add(1),
                y: pos.y,
            },
            CardinalDirection::West => Position {
                x: pos.x.saturating_sub(1),
                y: pos.y,
            },
        }
    }

    // TODO can we generate this with a macro?
    // or maybe custom serde?
    fn parse_command(msg: &str) -> Result<Command, String> {
        //info!("input:_{msg}_");
        match msg {
            "MOVE" => Ok(Command::MOVE),
            "DROP" => Ok(Command::DROP),
            "SCAN" => Ok(Command::SCAN),
            "STAT" => Ok(Command::STAT),
            "TURN L" => Ok(Command::TURN(Direction::L)),
            "TURN R" => Ok(Command::TURN(Direction::R)),
            "CHRG" => Ok(Command::CHRG),
            "BULD" => Ok(Command::BULD),
            x if x.starts_with("PICK") => {
                let kind = x.split_whitespace().nth(1).unwrap_or_default();
                //let kind = split.nth(1).unwrap_or_default();
                if let Some(prop) = Properties::from_user_input(kind) {
                    Ok(Command::PICK(prop))
                } else {
                    Err(format!("unknown entity {kind}"))
                }
            }
            _ => Err(format!("unknow command: {msg}")),
        }
    }
}

pub enum Direction {
    L,
    R,
}

#[allow(clippy::upper_case_acronyms)]
enum Command {
    TURN(Direction),
    PICK(Properties),
    MOVE,
    DROP,
    BULD,
    SCAN,
    CHRG,
    STAT,
}

impl Command {
    fn energy_cost(&self) -> usize {
        match self {
            Self::BULD => 10,
            Self::TURN(_) => 5,
            Self::PICK(_) => 5,
            Self::MOVE => 5,
            Self::DROP => 3,
            Self::SCAN => 3,
            Self::CHRG => 1,
            Self::STAT => 2,
        }
    }
}

#[derive(Debug)]
pub enum Reply {
    ERRR(String),
    DONE,
    BUSY,
    AREA(String),
    STAT {
        facing: CardinalDirection,
        position: Position,
        battery: usize,
        payload: String,
    },
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reply::ERRR(msg) => write!(f, "ERRR {msg}"),
            Reply::DONE => write!(f, "DONE"),
            Reply::BUSY => write!(f, "BUSY"),
            Reply::AREA(msg) => write!(f, "AREA {msg}"),
            Reply::STAT {
                facing,
                position,
                battery,
                payload,
            } => {
                let payload = if payload == " " {
                    &"_".to_string()
                } else {
                    payload
                };
                let battery_percent = (*battery as f64 / MAX_BATTERY as f64) * 100.0;
                write!(
                    f,
                    "STAT {} {} {} {:.0}% {}",
                    position.x, position.y, facing, battery_percent, payload
                )
            }
        }
    }
}

pub const DOCUMENTATION: &str = "DOG

  a mobile robot used to fetch resources and move components between machines

  equipped with a solar cell, DOG will slowly charge its battery when idle

COMMANDS

  MOVE
    move DOG one cordinate forward in the direction DOG is facing


  TURN direction={L|R}               
    turn DOG counterclockwise or clockwise

    Usage:
      TURN R ->  OKAY  #DOG turns clockwise

  
  SCAN
    return the three positions in front of DOG

    Usage:
      SCAN  ->  I..  #DOG is facing an IRON with two empty cordinates behind it
      SCAN  ->  III  #DOG is facing three IRON
      SCAN  ->  .II  #DOG is facing an empty cordinate followed by two IRON

  
  PICK entity={I|IRON|O|COPPER|...}    
    if DOG is facing `entity` or an agent containing `entity`, take it and place it in DOG's payload
    if DOG's payload is full, do nothing
  
    Usage:
      PICK I  #if DOG is facing IRON or an agent containing IRON, pick it and place in DOG's payload
      PICK IRON  #if DOG is facing IRON or an agent containing IRON, pick it and place in DOG's payload


  DROP
    drop DOG's payload
    if facing an agent, transfer DOG's payload to agent's BUFFER_IN


  BULD
    if DOG's payload contains a SOLAR_PANNEL, ACCUMULATOR, FABRICATOR, SMELTER, DOG, or LASER_CUTTER entity, attempt to build it on the surface in front of DOG

    the structure's footprint is built clockwise relative to DOG:

        .........................
        .>Fff.....v...Fff....Fff.
        ..fff...Fff...fff....fff.
        ..fff...fff...fff<...fff.
        ........fff..........ʌ...
        .........................

    Usage:
      BULD  ->  OKAY 3340                     #an agent was succesfully built and has port 3340
      BULD  ->  ERRR location not buildable   #structure footprint relative to DOG is not empty

  
  CHRG
    if DOG is facing an ACCUMULATOR, begin charging rapidly

    the next command sent to DOG will cancle charging 


  STAT
    return DOG's cordinates, the direction DOG is facing, battery percentage, and payload

    Usage:
      STAT  ->  STAT 10 20 E 85% _  #DOG is at (x=10, y=20) facing east, 85% battery, with no payload
      STAT  ->  STAT 20 10 N 3% R   #DOG is at (x=20, y=10) facing NORTH, 3% battery, carrying an IRON_PLATE
";
