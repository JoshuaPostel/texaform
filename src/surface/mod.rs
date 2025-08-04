use ratatui::Frame;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use tachyonfx::{Effect, fx};

use rand::Rng;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::agents::hud::{self, Hud};
use crate::agents::{Agent, Comms, UpdateEnum};
use crate::app::AppResult;
use crate::entities::Properties;
use crate::event::Event;
use crate::logging;
use crate::tech_tree::Tech;
use crate::tech_tree::TechTree;
use crate::ui::render_effect_clamped;
use crate::utils::pos_to_idx;

use petgraph::graph::NodeIndex;
use thiserror::Error;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use serde_with::serde_as;

pub mod generation;
pub mod grid;
pub mod tutorial;
use crate::surface::grid::{Gent, Grid};
use crate::surface::tutorial::Tutorial;

//const GRID_SIZE: usize = 1000;
const GRID_SIZE: usize = 250;

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Position(Position),
    Agent(usize),
}

#[serde_as]
#[derive(Serialize)]
pub struct Surface {
    pub x: usize,
    pub y: usize,
    pub grid: Grid,

    /// address to grid position mapping
    pub agents: BTreeMap<usize, Comms>,
    pub power: Power,

    //pub game_state: Arc<RwLock<GameState>>,
    pub game_state: GameState,

    pub game_stats: GameStats,
    pub victory_stats: Option<VictoryStats>,

    #[serde(skip)]
    pub hud: Hud,

    #[serde(skip)]
    pub focus: Option<Focus>,

    // TODO move this back up to App and use a reference/smart pointer?
    #[serde(skip)]
    pub event_sender: UnboundedSender<Event>,

    #[serde(skip)]
    effects: Vec<(Effect, Rect)>,
}

// order of fields matters for saving/loading
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct SurfaceState {
    x: usize,
    y: usize,
    pub grid: Grid,

    pub agents: BTreeMap<usize, Comms>,
    pub power: Power,
    pub game_state: GameState,
    pub game_stats: GameStats,
    pub victory_stats: Option<VictoryStats>,
}

impl SurfaceState {
    pub fn load(path: &std::path::Path) -> AppResult<SurfaceState> {
        //        let mut save_path = crate::logging::get_data_dir();
        //        save_path.push("save_file.texaform");
        let save_file = File::open(path)?;
        tracing::info!("save_file {save_file:?} opened");
        let mut reader = BufReader::new(save_file);
        tracing::info!("reader created");
        // TODO panics when deserializing different version of SurfaceState
        // need to set limit on how much it can read
        let surface_state: SurfaceState = bincode::deserialize_from(&mut reader)?;
        tracing::info!("loading worked!");
        //tracing::info!("{:?}", surface_state);
        Ok(surface_state)
    }

    pub async fn into_surface(mut self, event_sender: UnboundedSender<Event>) -> Surface {
        // TODO
        //        let game_state = Arc::new(RwLock::new(GameState {
        //            unlocked_entities: HashSet::from([Properties::Warehouse, Properties::Smelter]),
        //            tech_tree: TechTree::new(),
        //        }));
        for comms in self.agents.values_mut() {
            comms.init(&event_sender).await;
        }

        Surface {
            x: self.x,
            y: self.y,
            grid: self.grid,
            agents: self.agents,
            power: self.power,
            game_state: self.game_state,
            game_stats: self.game_stats,
            victory_stats: self.victory_stats,
            event_sender: event_sender.clone(),
            effects: vec![],
            focus: None,
            hud: Hud::default(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
/// use Joules for units
pub struct Power {
    // TODO make power.solar_pannel count and then calcualte
    // generation = solar_panel * solar_irradiance
    pub generation: usize,
    pub consumption: usize,
    pub stored: usize,
    pub capacity: usize,
    pub generation_history: [u64; 25],
    pub consumption_history: [u64; 25],
    pub solar_pannel_count: usize,
    //update_generation: usize,
    update_consumption: usize,
}

impl Power {
    //    pub fn generate(&mut self, joules: usize) {
    //        self.update_generation = self.update_generation.saturating_add(joules);
    //    }
    pub fn consume(&mut self, joules: usize) -> bool {
        self.update_consumption = self.update_consumption.saturating_add(joules);
        self.update_consumption < (self.generation + self.stored)
    }
    pub fn add_capacity(&mut self, joules: usize) {
        self.capacity = self.capacity.saturating_add(joules);
    }
    fn update(&mut self, solar_irradiance: usize) {
        self.generation = self.solar_pannel_count * solar_irradiance;
        self.consumption = self.update_consumption;
        //let net = self.update_generation as isize - self.update_consumption as isize;
        self.consumption_history.rotate_left(1);
        self.consumption_history[24] = self.update_consumption as u64;
        self.generation_history.rotate_left(1);
        //self.generation_history[24] = self.update_generation as u64;
        self.generation_history[24] = self.generation as u64;
        self.stored = self.stored.saturating_add(self.generation);
        self.stored = self.stored.saturating_sub(self.consumption);
        self.stored = self.stored.min(self.capacity);
        //self.update_generation = 0;
        self.update_consumption = 0;
    }
}

//TODO: add pos, ent to error
#[derive(Error, Debug)]
pub enum AddEntityError {
    #[error("Occupied")]
    Occupied,
    #[error("OutOfBounds")]
    OutOfBounds,
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.grid))
    }
}

impl Surface {
    pub fn focused_agent_port(&self) -> Option<usize> {
        match self.focus {
            Some(Focus::Agent(port)) => Some(port),
            _ => None,
        }
    }

    // TODO how to return Option<&dyn Agent>
    pub fn focused_agent(&self) -> Option<&Box<dyn Agent>> {
        self.focused_agent_port()
            .and_then(|port| self.get_agent(&port))
    }

    pub fn focused_agent_comms(&self) -> Option<&Comms> {
        self.focused_agent_port()
            .and_then(|port| self.agents.get(&port))
    }

    pub fn focused_agent_mut(&mut self) -> Option<&mut Box<dyn Agent>> {
        self.focused_agent_port()
            .and_then(|port| self.get_mut_agent(&port))
    }

    pub fn save(&self, name: String) {
        let mut save_path = crate::logging::get_data_dir();
        save_path.push(name + ".texaform");
        tracing::info!("saving to: {save_path:?}");
        // TODO handle this error properly
        let save_file = File::create(save_path).expect("Unable to create file");
        let writer = BufWriter::new(save_file);
        //bincode::serialize_into(writer, &self).unwrap();
        // TODO issue is here.  Why is writer writing default/empty self?
        //serde_json::to_writer(writer, &self).unwrap();
        tracing::info!("before save");
        bincode::serialize_into(writer, &self).unwrap();
        tracing::info!("after save");
    }

    pub fn tick(&mut self) {
        if self.game_state.tech_tree.victory_achieved && self.victory_stats.is_none() {
            let mut agent_count = HashMap::<String, u64>::new();
            for comms in self.agents.values() {
                *agent_count.entry(comms.entity.to_string()).or_insert(0) += 1;
            }
            self.victory_stats = Some(VictoryStats {
                seed: self.game_stats.seed,
                tick_count: self.game_stats.tick_count,
                manual_command_count: self.game_stats.manual_command_count,
                tcp_command_count: self.game_stats.tcp_command_count,
                agent_count,
                show_victory: true,
            });
        }
        self.game_stats.tick_count += 1;
        let positions: Vec<Position> = self
            .agents
            .values()
            .filter_map(|comms| comms.position)
            .collect();
        for pos in positions {
            if let Some(Gent::Age(agent)) = self.grid.get_mut(&pos) {
                agent.tick(&mut self.power);
            }
        }
        // TODO make solar_irradiance sinusoidal based on tick_count
        let solar_irradiance = 400;
        self.power.update(solar_irradiance);

        let mut agents_to_delete = vec![];
        for comms in self.agents.values() {
            if let Some(pos) = comms.position
                && let Some(Gent::Age(age)) = self.grid.get(&pos)
                && age.integrity() == 0
            {
                agents_to_delete.push(comms.port);
                let area = match age.properties().footprint() {
                    Some(fp) => Rect {
                        x: pos.x,
                        y: pos.y,
                        width: fp.x,
                        height: fp.y,
                    },
                    None => Rect {
                        x: pos.x,
                        y: pos.y,
                        width: 1,
                        height: 1,
                    },
                };
                // TODO better animation and move/abstract elsewhere
                // TODO impl as struct implementing trait tachyonfx::Shader
                let init_state: Vec<char> = area
                    .positions()
                    .map(|_| match rand::rng().random_range(0..3) {
                        0 => 'x',
                        1 => 'y',
                        _ => 'z',
                    })
                    .collect();
                let effect = fx::effect_fn(init_state, 20000, |state, context, cell_iter| {
                    if !context.timer.done() {
                        for (idx, (_pos, cell)) in cell_iter.enumerate() {
                            if let Some(c) = state.get_mut(idx) {
                                cell.set_char(*c);
                                match rand::rng().random_range(0..25) {
                                    0 => *c = 'x',
                                    1 => *c = 'y',
                                    2 => *c = 'z',
                                    _ => (),
                                };
                            }
                        }
                    }
                });

                self.effects.push((effect, area));
            }
        }
        //tracing::info!("atd: {agents_to_delete:?}");
        for port in agents_to_delete {
            tracing::info!("atd: {port}");
            self.delete_agent(&port);
        }
    }

    pub fn next_available_port(&self) -> usize {
        *self.agents.keys().max().unwrap_or(&3332) + 1
    }

    pub fn frame_position(&self, grid_position: &Position) -> Option<Position> {
        let x = grid_position.x.checked_sub(self.x as u16);
        let y = grid_position.y.checked_sub(self.y as u16);
        match (x, y) {
            (Some(x), Some(y)) => Some(Position { x, y }),
            _ => None,
        }
    }

    pub fn grid_position(&self, frame_position: &Position) -> Position {
        Position {
            x: frame_position.x.saturating_add(self.x as u16),
            y: frame_position.y.saturating_add(self.y as u16),
        }
    }

    pub fn render_agent_fx(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        prev_tick: core::time::Duration,
    ) {
        for comms in self.agents.values() {
            if let Some(pos) = comms.position
                && let Some(pos) = self.frame_position(&pos)
                && let Some(Gent::Age(agent)) = self.grid.get_mut(&pos)
            {
                agent.render_fx(&pos, frame, area, prev_tick)
            }
        }
        for (effect, fx_area) in self.effects.iter_mut() {
            let x = fx_area.x.checked_sub(self.x as u16);
            let y = fx_area.y.checked_sub(self.y as u16);
            let frame_pos = match (x, y) {
                (Some(x), Some(y)) => Some(Position { x, y }),
                _ => None,
            };
            if let Some(frame_pos) = frame_pos {
                let frame_area = Rect {
                    x: frame_pos.x,
                    y: frame_pos.y,
                    width: fx_area.width,
                    height: fx_area.height,
                };
                render_effect_clamped(frame, effect, frame_area, prev_tick)
            }
        }
    }

    pub fn render_grid(&self, frame: &mut Frame, area: Rect) {
        let no_offset = Position { x: 0, y: 0 };
        let clamped_area = area.clamp(frame.area());
        let buf = frame.buffer_mut();
        for pos in clamped_area.positions() {
            let cell = &mut buf[(pos.x, pos.y)];
            let grid_pos = self.grid_position(&pos);
            //tracing::info!("grid_pos: {grid_pos:?}");
            match self.grid.get_direct(&grid_pos) {
                None => {
                    cell.set_char(' ');
                    cell.bg = Color::Red;
                }
                Some(Gent::Intmd(p)) => {
                    cell.set_char(p.character());
                    cell.bg = p.bg();
                    cell.fg = p.fg();
                }
                Some(Gent::Age(agent)) => {
                    agent.render_surface_cell(&no_offset, cell);
                }
                // TODO just take props and render lowercase
                Some(Gent::Large(ent_pos)) => {
                    if let (Some(x), Some(y)) = (
                        grid_pos.x.checked_sub(ent_pos.x),
                        grid_pos.y.checked_sub(ent_pos.y),
                    ) {
                        let offset = Position::new(x, y);
                        match self.grid.get_direct(ent_pos) {
                            Some(Gent::Age(agent)) => agent.render_surface_cell(&offset, cell),
                            Some(Gent::Intmd(p)) => {
                                cell.set_char(p.character().to_ascii_lowercase());
                                cell.bg = p.bg();
                                cell.fg = p.fg();
                            }
                            _ => (),
                        }
                    }
                }
                Some(Gent::Empty) => {
                    // TODO not efficent
                    // should do once on startup, but not every render
                    // could just paint everyting black once in main before loop?
                    // and on resizing?
                    cell.bg = Color::Black;
                }
                Some(Gent::BeingUpdated) => {
                    tracing::error!("this code should not be reached");
                }
            }
        }
    }

    // TODO
    pub fn render_effects(
        &mut self,
        _frame: &mut Frame,
        _area: Rect,
        _prev_tick: std::time::Duration,
    ) {
    }

    pub fn add_entity(&mut self, pos: &Position, prop: Properties) -> Result<(), AddEntityError> {
        let idx = pos_to_idx(pos, GRID_SIZE);
        match prop.footprint() {
            Some(fp) => {
                let rect = Rect {
                    x: pos.x,
                    y: pos.y,
                    width: fp.x,
                    height: fp.y,
                };
                if self.grid.buildable(rect) {
                    prop.on_attach_to_power_grid(&mut self.power);
                    self.grid.insert(pos, Gent::Intmd(prop));
                    for pos in rect.positions().skip(1) {
                        self.grid
                            .insert(&pos, Gent::Large(Surface::idx_to_pos(idx)))
                    }
                    Ok(())
                } else {
                    tracing::error!("occupied idx: {}, {}", idx, prop);
                    Err(AddEntityError::Occupied)
                }
            }
            None => match self.grid.get(pos) {
                None => Err(AddEntityError::OutOfBounds),
                Some(Gent::Empty) => {
                    prop.on_attach_to_power_grid(&mut self.power);
                    self.grid.insert(pos, Gent::Intmd(prop));
                    Ok(())
                }
                _ => {
                    tracing::error!("occupied idx: {}, {}", idx, prop);
                    Err(AddEntityError::Occupied)
                }
            },
        }
    }

    pub async fn add_agent(
        &mut self,
        pos: &Position,
        agent: Box<dyn Agent + 'static>,
    ) -> Result<usize, AddEntityError> {
        let idx = pos_to_idx(pos, GRID_SIZE);
        let entity = agent.properties();
        match agent.properties().footprint() {
            Some(fp) => {
                let rect = Rect {
                    x: pos.x,
                    y: pos.y,
                    width: fp.x,
                    height: fp.y,
                };
                if self.grid.buildable(rect) {
                    self.grid.insert(pos, Gent::Age(agent));
                    let comms = Comms::new(self, Some(rect), entity).await;
                    let port = comms.port;
                    self.agents.insert(port, comms);
                    for pos in rect.positions().skip(1) {
                        self.grid
                            .insert(&pos, Gent::Large(Surface::idx_to_pos(idx)))
                    }
                    Ok(port)
                } else {
                    tracing::error!("occupied idx: {}, {}", idx, entity);
                    Err(AddEntityError::Occupied)
                }
            }
            None => {
                tracing::info!("NONE");
                match self.grid.get(pos) {
                    None => Err(AddEntityError::OutOfBounds),
                    Some(Gent::Empty) => {
                        self.grid.insert(pos, Gent::Age(agent));
                        // TODO should width and height be 0?
                        let location = Rect {
                            x: pos.x,
                            y: pos.y,
                            width: 1,
                            height: 1,
                        };
                        let comms = Comms::new(self, Some(location), entity).await;
                        let port = comms.port;
                        self.agents.insert(port, comms);
                        Ok(port)
                    }
                    _ => {
                        tracing::error!("occupied idx: {}, {}", idx, entity);
                        Err(AddEntityError::Occupied)
                    }
                }
            }
        }
    }

    // TODO could panic
    fn idx_to_pos(idx: usize) -> Position {
        Position {
            x: (idx % GRID_SIZE) as u16,
            y: (idx / GRID_SIZE) as u16,
        }
    }

    pub fn delete_agent(&mut self, port: &usize) {
        if self.focus == Some(Focus::Agent(*port)) {
            self.focus = None;
        }
        if let Some(pos) = self.agents.get(port).and_then(|comms| comms.position)
            && let Some(agent) = self.grid.pop(&pos)
        {
            let area = match agent.properties().footprint() {
                Some(fp) => Rect {
                    x: pos.x,
                    y: pos.y,
                    width: fp.x,
                    height: fp.y,
                },
                None => Rect {
                    x: pos.x,
                    y: pos.y,
                    width: 1,
                    height: 1,
                },
            };
            for pos in area.positions() {
                self.grid.insert(&pos, Gent::Empty);
            }
        }
        self.agents.remove(port);
    }

    // TODO is a borrowed box nessisary?
    #[allow(clippy::borrowed_box)]
    pub fn get_agent(&self, port: &usize) -> Option<&Box<dyn Agent>> {
        if let Some(pos) = self.agents.get(port).and_then(|comms| comms.position) {
            match &self.grid.get(&pos) {
                Some(Gent::Age(agent)) => return Some(agent),
                _ => return None,
            }
        }
        None
    }

    pub fn get_mut_agent(&mut self, port: &usize) -> Option<&mut Box<dyn Agent>> {
        if let Some(pos) = self.agents.get(port).and_then(|comms| comms.position) {
            match self.grid.get_mut(&pos) {
                Some(Gent::Age(agent)) => return Some(agent),
                _ => return None,
            }
        }
        None
    }

    // TODO is a borrowed box nessisary?
    //    #[allow(clippy::borrowed_box)]
    //    pub fn agents(&self) -> Vec<&Box<dyn Agent>> {
    //        let mut agents = vec![];
    //        for comms in self.agents.values() {
    //            if let Some(Gent::Age(agent)) = &self.grid.get(&comms.position) {
    //                agents.push(agent);
    //            }
    //        }
    //        agents
    //        // how to do like this?
    //        //self.agents.values().into_iter().map(|idx| &self.grid[*idx]).collect()
    //    }

    pub fn agent_port(&self, pos: &Position) -> Option<usize> {
        self.agents
            .iter()
            .find(|(_, v)| v.location.map(|l| l.contains(*pos)).unwrap_or(false))
            .map(|(k, _)| *k)
    }

    pub fn agent_position(&self, port: &usize) -> Option<Position> {
        self.agents.get(port).and_then(|comms| comms.position)
    }

    pub async fn update_agent_remote(&mut self, port: &usize, msg: String) {
        self.game_stats.tcp_command_count += 1;
        // TODO use unsafe mem::swap directly avoiding the push and pop
        // see source code of swap(a, b)
        let reply = if let Some(pos) = self.agent_position(port) {
            let gent = self.grid.pop(&pos);
            if let Some(Gent::Age(agent)) = gent {
                self.update_agent(msg.clone(), pos, port, agent).await
            } else {
                tracing::error!("expected agent at {pos:?}");
                return;
            }
        } else {
            self.handle_hud_command(msg.clone()).to_string()
        };
        if let Some(comms) = self.agents.get_mut(port) {
            comms.log.push((msg, reply.clone()));
            comms.sender().send(reply).await.expect("TODO");
        }
    }

    pub async fn update_agent_manual(&mut self, port: &usize) {
        self.game_stats.manual_command_count += 1;
        // TODO use unsafe mem::swap directly avoiding the push and pop
        // see source code of swap(a, b)
        if let Some(comms) = self.agents.get_mut(port) {
            let msg = comms.text_box.submit_message();
            let reply = if let Some(pos) = comms.position {
                let gent = self.grid.pop(&pos);
                if let Some(Gent::Age(agent)) = gent {
                    self.update_agent(msg.clone(), pos, port, agent).await
                } else {
                    tracing::error!("expected agent at {pos:?}");
                    return;
                }
            } else {
                self.handle_hud_command(msg.clone()).to_string()
            };
            if let Some(comms) = self.agents.get_mut(port) {
                comms.log.push((msg, reply));
            }
        } else {
            tracing::warn!("expected agent at port {port}");
        }
    }

    async fn update_agent(
        &mut self,
        msg: String,
        pos: Position,
        port: &usize,
        mut agent: Box<dyn Agent>,
    ) -> String {
        let update = agent.handle_message(&pos, &mut self.grid, &mut self.game_state, msg);
        match update {
            UpdateEnum::BuildAgent {
                pos: build_position,
                agent: new_agent,
            } => {
                self.grid.insert(&pos, Gent::Age(agent));
                match self.add_agent(&build_position, new_agent).await {
                    Ok(new_port) => format!("PORT {new_port}"),
                    Err(e) => {
                        tracing::error!("error adding agent: {e}");
                        // TODO do we want to panic?
                        panic!("error adding agent: {e}")
                    }
                }
            }
            UpdateEnum::BuildEntity {
                pos: build_position,
                entity,
            } => {
                self.grid.insert(&pos, Gent::Age(agent));
                match self.add_entity(&build_position, entity) {
                    Ok(_) => "OKAY".to_string(),
                    Err(e) => {
                        tracing::error!("error adding entity: {e}");
                        // TODO do we want to panic?
                        panic!("error adding agent: {e}")
                    }
                }
            }
            UpdateEnum::Move(new_position) => {
                tracing::info!("new_pos: {new_position}");
                let comms = self.agents.get_mut(port).expect("TODO need to Result");
                comms.position = Some(new_position);
                comms.location = Some(Rect {
                    x: new_position.x,
                    y: new_position.y,
                    width: comms.location.expect("moveable agent has width").width,
                    height: comms.location.expect("moveable agent has height").height,
                });
                self.grid.insert(&new_position, Gent::Age(agent));
                self.grid.insert(&pos, Gent::Empty);
                "OKAY".to_string()
            }
            UpdateEnum::Reply(reply) => {
                self.grid.insert(&pos, Gent::Age(agent));
                reply
            }
        }
    }

    pub fn move_up(&mut self, amount: usize) {
        let res = self.y.saturating_sub(amount);
        if res < GRID_SIZE {
            self.y = res;
        }
    }

    pub fn move_down(&mut self, amount: usize) {
        let res = self.y.saturating_add(amount);
        if res < GRID_SIZE {
            self.y = res;
        }
    }

    pub fn move_right(&mut self, amount: usize) {
        let res = self.x.saturating_add(amount);
        if res < GRID_SIZE {
            self.x = res;
        }
    }

    pub fn move_left(&mut self, amount: usize) {
        self.x = self.x.saturating_sub(amount);
    }

    fn handle_hud_command(&mut self, msg: String) -> hud::Reply {
        match Hud::parse_command(&msg) {
            Ok(command) => match command {
                hud::Command::STAT_POWR => (&self.power).into(),
                hud::Command::LIST_AGNT => {
                    if let Some((port, _)) = self.agents.iter().nth(self.hud.agent_idx)
                        && let Some(agent) = self.get_agent(port)
                    {
                        let kind = agent.properties().to_string();
                        self.hud.agent_idx += 1;
                        hud::Reply::LIST_AGNT { port: *port, kind }
                    } else {
                        self.hud.agent_idx = 1;
                        hud::Reply::LIST_AGNT {
                            port: 3333,
                            kind: "HUD".to_string(),
                        }
                    }
                }
                hud::Command::LIST_RESR => {
                    if let Some((tech, status)) = self
                        .game_state
                        .tech_tree
                        .get_tech_and_status(self.hud.research_idx)
                    {
                        self.hud.research_idx += 1;
                        hud::Reply::from_tech(tech, status)
                    } else {
                        self.hud.research_idx = 1;
                        let (tech, status) = self
                            .game_state
                            .tech_tree
                            .get_tech_and_status(0)
                            .expect("root node exists");

                        hud::Reply::from_tech(tech, status)
                    }
                }
                hud::Command::RESR(tech_kind) => {
                    let idx = self
                        .game_state
                        .tech_tree
                        .tech_kind_idx(tech_kind)
                        .expect("all tech in tree");
                    match self.game_state.tech_tree.set_research(idx) {
                        Ok(_) => hud::Reply::RESR,
                        Err(e) => hud::Reply::ERRR(e),
                    }
                }
            },
            Err(e) => hud::Reply::ERRR(e),
        }
    }

    pub async fn new(
        grid: Grid,
        x: usize,
        y: usize,
        event_sender: UnboundedSender<Event>,
    ) -> Surface {
        let game_state = GameState {
            unlocked_entities: HashSet::from([Properties::Dog]),
            tech_tree: TechTree::new(),
            tutorial_state: Tutorial::Start,
        };

        let mut surface = Surface {
            grid,
            x,
            y,
            agents: BTreeMap::new(),
            event_sender,
            game_state,
            game_stats: GameStats::default(),
            victory_stats: None,
            power: Power::default(),
            effects: vec![],
            focus: None,
            hud: Hud::default(),
        };
        let comms = Comms::new(&surface, None, Properties::HUD).await;
        surface.agents.insert(3333, comms);
        surface
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Seed {
    Random(u64),
    Manual(u64),
}

impl Seed {
    pub fn value(&self) -> u64 {
        match self {
            Seed::Random(x) => *x,
            Seed::Manual(x) => *x,
        }
    }

    pub fn ui_string(&self) -> String {
        match self {
            Seed::Manual(x) => format!("Seeded: {x}"),
            Seed::Random(x) => format!("Random Seed: {x}"),
        }
    }

    pub fn append(&mut self, digit: u64) {
        let value = match self {
            Seed::Random(_) => 0,
            Seed::Manual(x) => *x,
        };
        // ensure max seed of 999_999
        if value < 100_000 {
            *self = Seed::Manual((value * 10) + digit);
        }
    }

    pub fn backspace(&mut self) {
        if let Seed::Manual(value) = self {
            if 10 < *value {
                *self = Seed::Manual(*value / 10);
            } else {
                *self = Seed::default()
            }
        }
    }
}

impl Default for Seed {
    fn default() -> Seed {
        let mut rng = rand::rng();
        let seed = rng.random_range(0..999_999);
        Seed::Random(seed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    pub seed: Seed,
    pub tick_count: u64,
    pub manual_command_count: u64,
    pub tcp_command_count: u64,
    // TODO will require reworking Update
    // pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VictoryStats {
    pub seed: Seed,
    pub tick_count: u64,
    pub manual_command_count: u64,
    pub tcp_command_count: u64,
    // TODO will require reworking Update
    // pub error_count: u64,
    pub agent_count: HashMap<String, u64>,
    pub show_victory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    pub unlocked_entities: HashSet<Properties>,
    pub tech_tree: TechTree,
    pub tutorial_state: Tutorial,
}

impl GameState {
    //    pub fn progress_tech(&mut self, tech_node: usize) {
    //        if let Some(unlocked_entity) = self.tech_tree.progress(tech_node) {
    //            self.unlocked_entities.insert(unlocked_entity);
    //        }
    //    }

    pub fn progress_current_tech(&mut self) {
        if let Some(tech_node) = self.tech_tree.research_node {
            if let Some(unlocked_entity) = self.tech_tree.progress(tech_node) {
                self.unlocked_entities.insert(unlocked_entity);
            }
        }
    }

    pub fn current_tech(&self) -> Option<&Tech> {
        self.tech_tree
            .research_node
            .and_then(|node_idx| self.tech_tree.graph.node_weight(NodeIndex::new(node_idx)))
    }
}
