use rand::Rng;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::agents::Comms;
use crate::agents::hud::Hud;
use crate::app::AppResult;
use crate::entities::Entity;
use crate::event::Event;
use crate::tech_tree::Tech;
use crate::tech_tree::TechTree;
use crate::utils::human_readable_tick_count;

use petgraph::graph::NodeIndex;

use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::PathBuf;

use serde_with::serde_as;

use crate::surface::grid::Grid;
use crate::surface::tutorial::Tutorial;
use crate::surface::{Power, Surface};

// order of fields matters for saving/loading
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct SurfaceState {
    version: Version,
    x: usize,
    y: usize,
    pub grid: Grid,

    pub agents: BTreeMap<usize, Comms>,
    pub power: Power,
    pub game_state: GameState,
    pub victory_stats: Option<VictoryStats>,
}

#[serde_as]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

// update upon release
pub const VERSION: Version = Version {
    major: 0,
    minor: 1,
    patch: 0,
};

impl Default for Version {
    fn default() -> Version {
        VERSION
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    fn load(path: &std::path::Path) -> AppResult<Version> {
        let save_file = File::open(path)?;
        tracing::info!("save_file {save_file:?} opened");
        let mut reader = BufReader::new(save_file);
        tracing::info!("reader created");
        // std::mem::size_of::<Version>() == 3 (3 `u8`s)
        let config = bincode::config::standard().with_limit::<3>();
        let version: Version = bincode::serde::decode_from_std_read(&mut reader, config)?;
        tracing::info!("loaded version: {version}");
        Ok(version)
    }

    fn compatible(&self) -> bool {
        self.major == VERSION.major
    }
}

impl SurfaceState {
    pub fn save(surface: &Surface, name: String) -> AppResult<PathBuf> {
        let mut save_path = crate::logging::get_data_dir();
        save_path.push(name + ".texaform");
        tracing::info!("saving to: {save_path:?}");
        let save_file = File::create(save_path.clone())?;
        let mut writer = BufWriter::new(save_file);
        tracing::info!("before save");
        let config = bincode::config::standard().with_limit::<1_000_000>();
        bincode::serde::encode_into_std_write(surface, &mut writer, config).unwrap();
        tracing::info!("after save");
        Ok(save_path)
    }

    pub fn load(path: &std::path::Path) -> AppResult<SurfaceState> {
        let save_file = File::open(path)?;
        tracing::info!("save_file {save_file:?} opened");
        let mut reader = BufReader::new(save_file);
        tracing::info!("reader created");
        let version = Version::load(path)?;
        if !version.compatible() {
            return Err(format!(
                "save file format {version} is incompatible with texaform version {VERSION}"
            )
            .into());
        }
        let config = bincode::config::standard().with_limit::<1_000_000>();
        let surface_state = bincode::serde::decode_from_std_read(&mut reader, config)?;
        tracing::info!("loading worked!");
        Ok(surface_state)
    }

    pub async fn into_surface(mut self, event_sender: UnboundedSender<Event>) -> Surface {
        for comms in self.agents.values_mut() {
            comms.init(&event_sender).await;
        }

        Surface {
            version: self.version,
            x: self.x,
            y: self.y,
            grid: self.grid,
            agents: self.agents,
            power: self.power,
            game_state: self.game_state,
            victory_stats: self.victory_stats,
            event_sender: event_sender.clone(),
            effects: vec![],
            focus: None,
            hud: Hud::default(),
        }
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
            Seed::Manual(x) => format!("manual seed: {x}"),
            Seed::Random(x) => format!("random seed: {x}"),
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
    pub version: Version,
    pub seed: Seed,
    pub tick_count: u64,
    pub manual_command_count: u64,
    pub tcp_command_count: u64,
    pub research_complete: usize,
    pub research_count: usize,
    pub agent_count: BTreeMap<String, u64>,
    // TODO will require reworking Update
    // pub error_count: u64,
}

impl std::fmt::Display for GameStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let total_agent_count: u64 = self.agent_count.values().sum();
        let playtime = human_readable_tick_count(self.tick_count);
        write!(
            f,
            "version: {}\n{}\nplaytime: {}\nautomated commands: {}\nmanual commands: {}\ntechnology: {}/{}\ntotal agents: {}",
            self.version,
            self.seed.ui_string(),
            playtime,
            self.tcp_command_count,
            self.manual_command_count,
            self.research_complete,
            self.research_count,
            total_agent_count,
        )?;
        for (agent, count) in self.agent_count.iter() {
            write!(f, "\n  {agent}: {count}  ")?;
        }
        Ok(())
    }
}

impl GameStats {
    pub fn new(seed: Seed, research_count: usize) -> GameStats {
        GameStats {
            version: VERSION,
            seed,
            research_count,
            tick_count: 0,
            manual_command_count: 0,
            tcp_command_count: 0,
            research_complete: 0,
            agent_count: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VictoryStats {
    pub stats: GameStats,
    pub show_victory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameState {
    pub unlocked_entities: HashSet<Entity>,
    pub tech_tree: TechTree,
    pub tutorial_state: Tutorial,
    pub stats: GameStats,
}

impl GameState {
    pub fn progress_current_tech(&mut self) {
        if let Some(tech_node) = self.tech_tree.research_node
            && let Some(unlocked_entity) = self.tech_tree.progress(tech_node)
        {
            self.unlocked_entities.insert(unlocked_entity);
            self.stats.research_complete += 1;
        }
    }

    pub fn current_tech(&self) -> Option<&Tech> {
        self.tech_tree
            .research_node
            .and_then(|node_idx| self.tech_tree.graph.node_weight(NodeIndex::new(node_idx)))
    }
}
