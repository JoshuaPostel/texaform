use std::collections::HashMap;

use crate::agents::{Agent, UpdateEnum};
use crate::entities::{Entity, EntityContainer, PickResult};
use crate::surface::grid::Grid;
use crate::surface::state::GameState;

use serde::{Deserialize, Serialize};

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::widgets::WidgetRef;

impl WidgetRef for Fabricator {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::horizontal([
            Constraint::Max(22),
            Constraint::Fill(1),
            Constraint::Max(22),
        ])
        .split(area);
        self.buffer_in.render_ref(chunks[0], buf);
        self.buffer_out.render_ref(chunks[2], buf);

        // TODO print counted summary too
        //        let mut counter = BTreeMap::<char, usize>::new();
        //        for c in self.content.iter().map(|ent| ent.character()) {
        //            counter
        //                .entry(c)
        //                .and_modify(|count| *count += 1)
        //                .or_insert(1);
        //        }
        //        let mut test = String::new();
        //        for c in &self.content {
        //            test.push(c.character());
        //        }
        //
        //        let p = Paragraph::new(format!("content: {:?}\ntesting: {}", counter, test));
        //        p.render_ref(area, buf);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fabricator {
    pub buffer_in: EntityContainer,
    pub buffer_out: EntityContainer,
}

#[typetag::serde]
impl Agent for Fabricator {
    fn new() -> Self {
        Self {
            buffer_in: EntityContainer::new("BUFFER_IN", 20),
            buffer_out: EntityContainer::new("BUFFER_OUT", 20),
        }
    }
    fn handle_message(
        &mut self,
        pos: &Position,
        grid: &mut Grid,
        game_state: &mut GameState,
        msg: String,
    ) -> UpdateEnum {
        match Self::parse_command(&msg) {
            Ok(command) => self.handle_command(command, pos, grid, game_state),
            Err(e) => UpdateEnum::reply(format!("ERRR: {e}")),
        }
    }

    fn entity(&self) -> Entity {
        Entity::Fabricator
    }
    fn pick(&mut self, c: char) -> PickResult {
        let buffer_out_pr = self.buffer_out.pick(c);
        if buffer_out_pr.picked.is_some() {
            buffer_out_pr
        } else {
            self.buffer_in.pick(c)
        }
    }

    fn placable(&self, _entity: &Entity) -> bool {
        self.buffer_in.placable()
    }
    fn place(&mut self, entity: Entity) {
        self.buffer_in.place(entity)
    }
}

fn contains_cost(buffer: &[Entity], cost: &HashMap<Entity, u8>) -> bool {
    cost.iter()
        .all(|(entity, count)| *count <= buffer.iter().filter(|&p| p == entity).count() as u8)
}

impl Fabricator {
    fn handle_command(
        &mut self,
        command: Command,
        _pos: &Position,
        _grid: &mut Grid,
        game_state: &mut GameState,
    ) -> UpdateEnum {
        match command {
            Command::STAT => UpdateEnum::reply(Reply::STAT {
                content: self
                    .buffer_in
                    .content
                    .iter()
                    .map(|p| p.character())
                    .collect(),
            }),
            Command::RESR => {
                if let Some(tech) = game_state.current_tech() {
                    if contains_cost(&self.buffer_in.content, &tech.cost) {
                        self.buffer_in
                            .remove_content(&tech.cost)
                            .expect("checked previously");
                        game_state.progress_current_tech();
                        UpdateEnum::okay()
                    } else {
                        UpdateEnum::reply(Reply::ERRR("insufficent materials".to_string()))
                    }
                } else {
                    UpdateEnum::reply(Reply::ERRR("no active research".to_string()))
                }
            }
            Command::MAKE(entity) => {
                let kind_unlocked = game_state.unlocked_entities.contains(&entity);
                let cost = &entity
                    .cost()
                    .expect("checked existance when parsing the command");
                let contains_materials = contains_cost(&self.buffer_in.content, cost);
                if !kind_unlocked {
                    UpdateEnum::reply(Reply::ERRR("not unlocked".to_string()))
                } else if !contains_materials {
                    UpdateEnum::reply(Reply::ERRR("insufficent materials".to_string()))
                } else {
                    self.buffer_in.remove_content(cost).expect("CHECKED");
                    self.buffer_out.place(entity);
                    UpdateEnum::okay()
                }
            }
        }
    }

    fn parse_command(msg: &str) -> Result<Command, String> {
        match msg {
            x if x.starts_with("MAKE") => {
                let kind = x.split_whitespace().nth(1).unwrap_or_default();
                if let Some(entity) = Entity::from_user_input(kind) {
                    Ok(Command::MAKE(entity))
                } else {
                    Err(format!("unknown entity {kind}"))
                }
            }
            "RESR" => Ok(Command::RESR),
            "STAT" => Ok(Command::STAT),
            _ => Err(format!("unknown command: {msg}")),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
enum Command {
    MAKE(Entity),
    RESR,
    STAT,
}

pub const DOCUMENTATION: &str = "FABRICATOR

  consumes entities to conduct research or build entities/agents

COMMANDS

  RESR
    if a research is selected and FABRICATOR contains the research's cost, progress the research by 1
  
  MAKE entity={DOG|SMELTER|MOTOR|ACCUMULATOR|...}    
    if FABRICATOR's buffer_in contains the entity's material cost, the materials will be consumed and the entity will added to FABRICATOR's buffer_out

    Usage:
      MAKE SMELTER  ->  OKAY
      MAKE MOTOR    ->  ERRR insufficent material

  STAT
    returns the contents of FABRICATOR

    Usage:
      STAT  ->  STAT _ _      #FABRICATOR's buffer_in and buffer_out are empty
      STAT  ->  STAT IIIOO _  #FABRICATOR's buffer_in contains 3 IRON and 2 COPPER
      STAT  ->  STAT _ MM     #FABRICATOR's buffer_out contains 2 MOTOR
";

#[derive(Debug)]
pub enum Reply {
    ERRR(String),
    ADDR(usize),
    STAT { content: Vec<char> },
}

impl std::fmt::Display for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Reply::ERRR(msg) => write!(f, "ERRR {msg}"),
            Reply::ADDR(a) => write!(f, "ADDR {a}"),
            Reply::STAT { content } => {
                write!(f, "STAT ")?;
                if content.is_empty() {
                    write!(f, "_")?;
                } else {
                    for c in content {
                        write!(f, "{c}")?;
                    }
                }
                Ok(())
            }
        }
    }
}
