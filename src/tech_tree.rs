use core::fmt;
use std::collections::{HashMap, HashSet};

use petgraph::algo::bellman_ford::bellman_ford;
use petgraph::graph::DiGraph;

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

use ratatui::layout::{Constraint, Flex, Layout, Margin, Position, Rect};
use strum_macros;

use crate::entities::Properties;
use crate::ui::tech_tree::EdgeLayout;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechTree {
    pub graph: DiGraph<Tech, f32>,
    // order/index matches graph index
    // depth is the max distance from root
    pub node_depths: Vec<usize>,
    pub selected_node: usize,
    pub research_node: Option<usize>,
    pub victory_achieved: bool,
    pub everything_researched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tech {
    pub kind: TechKind,
    pub cost: HashMap<Properties, u8>,
    pub progress_numerator: u8,
    pub progress_denominator: u8,
    pub unlocked: bool,
    pub unlocks: Option<Properties>,
}

// TODO rename Tech to TechNode?
#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    strum_macros::EnumString,
    strum_macros::EnumMessage,
    strum_macros::Display,
    strum_macros::AsRefStr,
)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum TechKind {
    Smelter,
    LaserCutter,
    SolarPannel,
    Battery,
    Fabricator,
    Dog,
    Accumulator,
    SelfSufficient,
}

impl Default for Tech {
    fn default() -> Tech {
        Tech::new(TechKind::Dog, HashMap::new(), None)
    }
}

impl Tech {
    pub fn new(kind: TechKind, cost: HashMap<Properties, u8>, unlocks: Option<Properties>) -> Tech {
        Tech {
            kind,
            cost,
            progress_numerator: 0,
            progress_denominator: 2,
            unlocked: false,
            unlocks,
        }
    }

    fn progress(&mut self) {
        if !self.unlocked {
            self.progress_numerator += 1;
            if self.progress_numerator == self.progress_denominator {
                self.unlocked = true;
            }
        }
    }
}

pub enum TechStatus {
    Researched,
    Selected,
    Unlocked,
    Locked,
}

impl std::fmt::Display for TechStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TechStatus::Researched => "RESEARCHED",
            TechStatus::Selected => "SELECTED",
            TechStatus::Unlocked => "UNLOCKED",
            TechStatus::Locked => "LOCKED",
        };
        write!(f, "{s}")
    }
}

impl TechTree {
    fn everything_unlocked(&self) -> bool {
        self.graph
            .raw_nodes()
            .iter()
            .all(|node| node.weight.unlocked)
    }

    pub fn unlocked_count(&self) -> usize {
        self.graph
            .raw_nodes()
            .iter()
            .filter(|node| node.weight.unlocked)
            .count()
    }

    pub fn get_tech_and_status(&self, idx: usize) -> Option<(&Tech, TechStatus)> {
        if let Some(tech) = self.get_tech(idx)
            && let Some(status) = self.get_tech_status(idx)
        {
            Some((tech, status))
        } else {
            None
        }
    }

    fn get_tech_status(&self, idx: usize) -> Option<TechStatus> {
        if let Some(research_node) = self.research_node
            && research_node == idx
        {
            return Some(TechStatus::Selected);
        }
        if let Some(tech) = self.get_tech(idx) {
            if tech.unlocked {
                Some(TechStatus::Researched)
            } else if self.prerequisits_unlocked(idx) {
                Some(TechStatus::Unlocked)
            } else {
                Some(TechStatus::Locked)
            }
        } else {
            None
        }
    }

    pub fn new() -> TechTree {
        let mut graph = DiGraph::new();

        let smelter = Tech {
            kind: TechKind::Smelter,
            cost: HashMap::from([(Properties::Iron, 1)]),
            progress_numerator: 0,
            progress_denominator: 2,
            unlocked: false,
            unlocks: Some(Properties::Smelter),
        };
        let root = graph.add_node(smelter);

        let laser_cutter = Tech {
            kind: TechKind::LaserCutter,
            cost: HashMap::from([(Properties::IronPlate, 1), (Properties::CopperPlate, 1)]),
            progress_numerator: 0,
            progress_denominator: 2,
            unlocked: false,
            unlocks: Some(Properties::LaserCutter),
        };
        graph.add_node(laser_cutter);

        graph.add_node(Tech::new(
            TechKind::SolarPannel,
            HashMap::from([(Properties::CopperPlate, 1), (Properties::Wafer, 1)]),
            Some(Properties::SolarPannel),
        ));
        graph.add_node(Tech::new(
            TechKind::Battery,
            HashMap::from([(Properties::IronPlate, 1), (Properties::Sulfer, 1)]),
            Some(Properties::Battery),
        ));
        let fabricator = Tech {
            kind: TechKind::Fabricator,
            cost: HashMap::from([(Properties::Nut, 2)]),
            progress_numerator: 0,
            progress_denominator: 4,
            unlocked: false,
            unlocks: Some(Properties::Fabricator),
        };
        graph.add_node(fabricator);
        graph.add_node(Tech::new(
            TechKind::Dog,
            HashMap::from([
                (Properties::Gear, 1),
                (Properties::Battery, 1),
                (Properties::SolarPannel, 1),
            ]),
            Some(Properties::Dog),
        ));
        let accumulator = Tech {
            kind: TechKind::Accumulator,
            cost: HashMap::from([(Properties::Battery, 2)]),
            progress_numerator: 0,
            progress_denominator: 4,
            unlocked: false,
            unlocks: Some(Properties::Accumulator),
        };
        graph.add_node(accumulator);
        let self_sufficent = Tech {
            kind: TechKind::SelfSufficient,
            cost: HashMap::from([(Properties::Dog, 1)]),
            progress_numerator: 0,
            progress_denominator: 10,
            unlocked: false,
            unlocks: None,
        };
        graph.add_node(self_sufficent);
        // negative distances for bellman_ford to give longest path
        graph.extend_with_edges([
            (0, 1, -1.0),
            (0, 2, -1.0),
            (0, 3, -1.0),
            (1, 4, -1.0),
            (1, 5, -1.0),
            (2, 5, -1.0),
            (3, 5, -1.0),
            (3, 6, -1.0),
            (5, 7, -1.0),
            // testing distances
            //(0, 5, -1.0),
        ]);

        // let distances = dijkstra(&graph, root, None, |_| 1);
        // tracing::info!("shortest: {distances:#?}");
        //
        // works without -1.0 float being stored,
        // but computes distance for every pair of nodes
        // graph.extend_with_edges(&[
        //     (0, 1), (0, 2), (0, 3),
        //     (1, 4), (2, 4), (2, 5), (3, 5),
        //     // testing distances
        //     (0, 5)
        //
        // ]);
        // let distances = floyd_warshall(&graph, |_| -1);
        // tracing::info!("longest: {distances:#?}");

        let paths = bellman_ford(&graph, root).expect("non-cyclic");
        //tracing::info!("longest: {paths:#?}");

        let node_depths = paths
            .distances
            .into_iter()
            .map(|f: f32| f.abs() as usize)
            .collect();
        tracing::info!("node_depths: {node_depths:#?}");

        TechTree {
            graph,
            node_depths,
            selected_node: 0,
            research_node: None,
            victory_achieved: false,
            everything_researched: false,
        }
    }

    pub fn get_tech(&self, idx: usize) -> Option<&Tech> {
        self.graph.node_weight(NodeIndex::new(idx))
    }

    pub fn selected_tech(&self) -> Option<&Tech> {
        self.graph.node_weight(NodeIndex::new(self.selected_node))
    }

    pub fn researching(&self) -> Option<&Tech> {
        self.research_node
            .and_then(|node_idx| self.graph.node_weight(NodeIndex::new(node_idx)))
    }

    pub fn researching_mut(&mut self) -> Option<&mut Tech> {
        self.research_node
            .and_then(|node_idx| self.graph.node_weight_mut(NodeIndex::new(node_idx)))
    }

    pub fn set_research(&mut self, node_index: usize) -> Result<(), String> {
        if let Some(tech) = self.graph.node_weight(NodeIndex::new(node_index)) {
            match (tech.unlocked, self.prerequisits_unlocked(node_index)) {
                (false, true) => {
                    self.research_node = Some(node_index);
                    Ok(())
                }
                (true, _) => Err("already researched".to_string()),
                (false, false) => Err("prerequisits not met".to_string()),
            }
        } else {
            Err("node not found".to_string())
        }
    }

    // PERF: this could be hardcoded because our graph does not change
    pub fn tech_kind_idx(&self, kind: TechKind) -> Option<usize> {
        self.graph
            .node_indices()
            .find(|i| self.graph[*i].kind == kind)
            .map(|n| n.index())
    }

    pub fn progress(&mut self, node_idx: usize) -> Option<Properties> {
        if Some(node_idx) == self.research_node
            && let Some(tech) = self.researching_mut()
        {
            tech.progress();
            if tech.unlocked {
                let unlocks = tech.unlocks;
                if tech.kind == TechKind::SelfSufficient {
                    self.victory_achieved = true;
                }
                if self.everything_unlocked() {
                    self.everything_researched = true;
                }
                self.research_node = None;
                return unlocks;
            }
        }
        None
    }

    pub fn prerequisits_unlocked(&self, node_index: usize) -> bool {
        let mut prerequisits = self
            .graph
            .neighbors_directed(NodeIndex::new(node_index), petgraph::Direction::Incoming)
            .detach();
        while let Some(idx) = prerequisits.next_node(&self.graph) {
            if !self.graph.node_weight(idx).expect("exists").unlocked {
                return false;
            }
        }
        true
    }

    fn row_areas(&self, row_idx: usize, rect: Rect) -> Vec<(usize, Rect)> {
        let node_indexes: Vec<usize> = self
            .node_depths
            .iter()
            .enumerate()
            .filter(|(_, depth)| depth == &&row_idx)
            .map(|(idx, _)| idx)
            .collect();

        let row = Layout::horizontal(vec![Constraint::Max(17); node_indexes.len()])
            .flex(Flex::SpaceAround)
            .split(rect);

        node_indexes
            .into_iter()
            .enumerate()
            .map(|(col, node_index)| (node_index, row[col]))
            .collect()
    }

    pub fn update_node_layout(&self, area: &Rect) -> Vec<Rect> {
        let paths = bellman_ford(&self.graph, NodeIndex::new(0)).expect("non-cyclic");
        //tracing::info!("longest: {paths:#?}");

        let node_depths: Vec<usize> = paths
            .distances
            .into_iter()
            .map(|f: f32| f.abs() as usize)
            .collect();

        let max_depth = node_depths.iter().max().expect("at least one element");

        let inner = area.inner(Margin::new(1, 1));
        let mut constraints: Vec<Constraint> = vec![];
        for _ in 0..=*max_depth {
            constraints.push(Constraint::Max(3));
            constraints.push(Constraint::Max(5));
        }
        let chunks = Layout::vertical(constraints).split(inner);

        // build and then order node_uis vec so that vector's idx matches graph node index
        let mut node_areas = vec![];
        for i in 0..=*max_depth {
            node_areas.extend(self.row_areas(i, chunks[i * 2]));
        }
        node_areas.sort_by_key(|(idx, _)| *idx);
        node_areas.into_iter().map(|(_, node_ui)| node_ui).collect()
    }

    pub fn update_edge_layout(&self, node_areas: &[Rect]) -> Vec<EdgeLayout> {
        let mut node_layouts: Vec<NodeLayout> = node_areas.iter().map(NodeLayout::new).collect();

        let mut edge_layouts = vec![];
        for edge in self.graph.raw_edges() {
            let source = &node_layouts[edge.source().index()];
            let target = &node_layouts[edge.target().index()];
            let edge_layout = get_edge_layout(source, target);
            node_layouts[edge.source().index()]
                .ocupied_terminals
                .insert(edge_layout.start);
            node_layouts[edge.target().index()]
                .ocupied_terminals
                .insert(edge_layout.end);
            edge_layouts.push(edge_layout);
        }

        edge_layouts
    }
}

#[derive(Clone)]
struct NodeLayout {
    area: Rect,
    ocupied_terminals: HashSet<Position>,
}

impl NodeLayout {
    fn new(area: &Rect) -> NodeLayout {
        NodeLayout {
            area: *area,
            ocupied_terminals: HashSet::new(),
        }
    }
}

fn get_edge_layout(start: &NodeLayout, end: &NodeLayout) -> EdgeLayout {
    let target_is_left = end.area.x < start.area.x;

    let mut start_position = bottom(start.area);
    loop {
        if start.ocupied_terminals.contains(&start_position) {
            if target_is_left {
                start_position.x -= 1;
            } else {
                start_position.x += 1;
            }
            continue;
        } else {
            break;
        }
    }
    let directly_down =
        end.area.x < start_position.x && start_position.x < (end.area.x + end.area.width - 1);
    let mut end_position = if directly_down {
        Position {
            x: start_position.x,
            y: end.area.y,
        }
    } else {
        top(end.area)
    };
    loop {
        if end.ocupied_terminals.contains(&end_position) {
            if target_is_left {
                end_position.x += 1;
            } else {
                end_position.x -= 1;
            }
            continue;
        } else {
            break;
        }
    }

    EdgeLayout {
        start: start_position,
        end: end_position,
    }
}

fn top(rect: Rect) -> Position {
    Position {
        x: rect.x + (rect.width / 2),
        y: rect.y,
    }
}

fn bottom(rect: Rect) -> Position {
    Position {
        x: rect.x + (rect.width / 2),
        y: rect.y + rect.height - 1,
    }
}

//#[derive(Debug, Default)]
//pub struct EdgeLayout {
//    pub start: Position,
//    pub end: Position,
//}
