use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Gauge, Paragraph, Widget, Wrap},
};

use petgraph::graph::NodeIndex;

use crate::app::App;
use crate::tech_tree::TechTree;
use crate::ui::render_widget_clamped;

#[derive(Debug, Default)]
pub struct TechTreeLayout {
    pub description: Rect,
    pub tree: Rect,
    pub current_research: Rect,
    pub nodes: Vec<Rect>,
    pub edges: Vec<EdgeLayout>,
}

#[derive(Debug, Default)]
pub struct EdgeLayout {
    pub start: Position,
    pub end: Position,
}

impl TechTreeLayout {
    pub fn new(width: u16, height: u16, app: &App) -> TechTreeLayout {
        let chunks = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(Rect {
                x: 0,
                y: 0,
                width,
                height,
            });
        let left_column =
            Layout::vertical([Constraint::Max(3), Constraint::Fill(1)]).split(chunks[0]);

        let tech_tree = &app.surface.game_state.tech_tree;
        let nodes = tech_tree.update_node_layout(&chunks[1]);
        let edges = tech_tree.update_edge_layout(&nodes);

        let description = left_column[1];

        TechTreeLayout {
            current_research: left_column[0],
            description,
            tree: chunks[1],
            nodes,
            edges,
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    render_current_research(app, frame, app.layout.tech_tree.current_research);
    render_research_description(app, frame, app.layout.tech_tree.description);

    render_tech_tree(app, frame, app.layout.tech_tree.tree);
    render_widget_clamped(
        frame,
        app.previous_screen_button.clone(),
        app.layout.previous_screen_button,
    );
}

pub fn render_tech_tree(app: &App, frame: &mut Frame, area: Rect) {
    let tech_tree = &app.surface.game_state.tech_tree;

    let border = Block::bordered()
        .title("Technology Tree")
        .style(Style::new().bg(Color::Black).fg(Color::Green));

    border.render(area, frame.buffer_mut());

    render_nodes(tech_tree, app, frame);
    render_edges(tech_tree, app, frame);
}

pub fn render_research_description(app: &App, frame: &mut Frame, _area: Rect) {
    let tech_tree = &app.surface.game_state.tech_tree;
    let (title, content) = match tech_tree.selected_tech() {
        Some(tech) => {
            let mut description = format!(
                "\nresearch count: {}\n\nresearch cost:\n",
                tech.progress_denominator
            );
            for (entity, count) in &tech.cost {
                description.push_str(&format!("  {entity}: {count}\n"));
            }
            if let Some(entity) = tech.unlocks {
                description.push_str(&format!("\nunlocks: {entity}\n"))
            }
            (format!(": {}", tech.kind), description)
        }
        // TODO this branch might be unreachable
        None => (
            "".to_string(),
            "click on a technology to learn about it".to_string(),
        ),
    };
    let paragraph = Paragraph::new(content)
        .block(Block::bordered().title(format!("Research Information{title}")))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, paragraph, app.layout.tech_tree.description);
}

pub fn current_research_content(app: &App) -> (String, Gauge) {
    match app.surface.game_state.tech_tree.researching() {
        None => {
            let label = if app.surface.game_state.tech_tree.everything_researched {
                "All research complete"
            } else {
                "Select a technology"
            };
            let paragraph = Gauge::default()
                .style(Style::default().fg(Color::Green).bg(Color::Black))
                .label(label);
            ("Nothing".to_string(), paragraph)
        }
        Some(tech) => {
            let guage = Gauge::default()
                .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                .label(format!(
                    "{}/{}",
                    tech.progress_numerator, tech.progress_denominator
                ))
                .percent(
                    (100.0 * f64::from(tech.progress_numerator)
                        / f64::from(tech.progress_denominator)) as u16,
                );
            (tech.kind.to_string(), guage)
        }
    }
}

pub fn render_current_research(app: &App, frame: &mut Frame, area: Rect) {
    //    let name = match app.surface.game_state.tech_tree.researching() {
    //        None => {
    //            let paragraph = Paragraph::new("Select a technology")
    //                .wrap(Wrap { trim: false })
    //                .style(Style::default().fg(Color::Green).bg(Color::Black));
    //            render_widget_clamped(frame, paragraph, area.inner(Margin::new(1, 1)));
    //            "Nothing"
    //        }
    //        Some(tech) => {
    //            let guage = Gauge::default()
    //                .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
    //                .label(format!(
    //                    "{}/{}",
    //                    tech.progress_numerator, tech.progress_denominator
    //                ))
    //                .percent(
    //                    (100.0 * f64::from(tech.progress_numerator)
    //                        / f64::from(tech.progress_denominator)) as u16,
    //                );
    //            render_widget_clamped(frame, guage, area.inner(Margin::new(1, 1)));
    //            &tech.name
    //        }
    //    };
    let (name, guage) = current_research_content(app);
    let block = Block::bordered()
        .title("Researching: ".to_string() + &name)
        .border_style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, block, area);
    render_widget_clamped(frame, guage, area.inner(Margin::new(1, 1)));
}

fn render_nodes(tech_tree: &TechTree, app: &App, frame: &mut Frame) {
    for (node_index, area) in app.layout.tech_tree.nodes.iter().enumerate() {
        if let Some(tech) = tech_tree.graph.node_weight(NodeIndex::new(node_index)) {
            let color = if tech.unlocked {
                Color::Green
            } else if Some(node_index) == tech_tree.research_node {
                Color::LightYellow
            } else if tech_tree.prerequisits_unlocked(node_index) {
                Color::Yellow
            } else {
                Color::Red
            };
            let style = if node_index == tech_tree.selected_node {
                Style::new().bg(color).fg(Color::Black)
            } else {
                Style::new().bg(Color::Black).fg(color)
            };
            let paragraph = Paragraph::new(tech.kind.to_string())
                .block(Block::bordered())
                .centered()
                .style(style);
            paragraph.render(*area, frame.buffer_mut());
        }
    }
}

fn render_edges(tech_tree: &TechTree, app: &App, frame: &mut Frame) {
    let edges = &app.layout.tech_tree.edges;
    let buf = frame.buffer_mut();
    for (edge_layout, edge) in edges.iter().zip(tech_tree.graph.raw_edges()) {
        let color = if tech_tree
            .graph
            .node_weight(NodeIndex::new(edge.target().index()))
            .expect("index exists")
            .unlocked
        {
            Color::Green
        } else if tech_tree
            .graph
            .node_weight(NodeIndex::new(edge.source().index()))
            .expect("index exists")
            .unlocked
        {
            Color::Yellow
        } else {
            Color::Red
        };
        render_edge(edge_layout, color, buf)
    }
}

fn render_edge(edge: &EdgeLayout, color: Color, buf: &mut Buffer) {
    let y_midpoint = edge.start.y + ((edge.end.y - edge.start.y) / 2);

    let cell = &mut buf[(edge.start.x, edge.start.y)];
    let d: Directions = cell.symbol().into();
    cell.set_char(d.add_south().into());
    // intentionally not coloring start
    // cell.set_fg(color);

    let cell = &mut buf[(edge.end.x, edge.end.y)];
    let d: Directions = cell.symbol().into();
    cell.set_char(d.add_north().into());
    // intentionally not coloring end
    //cell.set_fg(color);

    for y in (edge.start.y + 1)..y_midpoint {
        let cell = &mut buf[(edge.start.x, y)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_north().add_south().into());
        cell.set_fg(color);
    }
    for y in (y_midpoint + 1)..edge.end.y {
        let cell = &mut buf[(edge.end.x, y)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_north().add_south().into());
        cell.set_fg(color);
    }

    // TODO use cmp() and match?
    if edge.start.x == edge.end.x {
        let cell = &mut buf[(edge.start.x, y_midpoint)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_north().add_south().into());
        cell.set_fg(color);
    } else if edge.start.x < edge.end.x {
        let cell = &mut buf[(edge.start.x, y_midpoint)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_north().add_east().into());
        cell.set_fg(color);

        let cell = &mut buf[(edge.end.x, y_midpoint)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_south().add_west().into());
        cell.set_fg(color);

        for x in (edge.start.x + 1)..edge.end.x {
            let cell = &mut buf[(x, y_midpoint)];
            let d: Directions = cell.symbol().into();
            cell.set_char(d.add_east().add_west().into());
            cell.set_fg(color);
        }
    } else if edge.end.x < edge.start.x {
        let cell = &mut buf[(edge.start.x, y_midpoint)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_north().add_west().into());
        cell.set_fg(color);

        let cell = &mut buf[(edge.end.x, y_midpoint)];
        let d: Directions = cell.symbol().into();
        cell.set_char(d.add_south().add_east().into());
        cell.set_fg(color);

        for x in (edge.end.x + 1)..edge.start.x {
            let cell = &mut buf[(x, y_midpoint)];
            let d: Directions = cell.symbol().into();
            cell.set_char(d.add_east().add_west().into());
            cell.set_fg(color);
        }
    }
}

struct Directions {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

impl Directions {
    fn add_north(self) -> Self {
        Directions {
            north: true,
            ..self
        }
    }
    fn add_south(self) -> Self {
        Directions {
            south: true,
            ..self
        }
    }
    fn add_east(self) -> Self {
        Directions { east: true, ..self }
    }
    fn add_west(self) -> Self {
        Directions { west: true, ..self }
    }
}

impl From<(bool, bool, bool, bool)> for Directions {
    fn from(t: (bool, bool, bool, bool)) -> Directions {
        Directions {
            north: t.0,
            south: t.1,
            east: t.2,
            west: t.3,
        }
    }
}

impl From<&str> for Directions {
    fn from(s: &str) -> Directions {
        match s {
            // up-down
            "│" => (true, true, false, false).into(),
            "╵" => (true, false, false, false).into(),
            "╷" => (false, true, false, false).into(),
            // left-right
            "─" => (false, false, true, true).into(),
            "╴" => (false, false, true, false).into(),
            "╶" => (false, false, false, true).into(),
            // no north
            "┌" => (false, true, true, false).into(),
            "┐" => (false, true, false, true).into(),
            "┬" => (false, true, true, true).into(),
            // no south
            "└" => (true, false, true, false).into(),
            "┘" => (true, false, false, true).into(),
            "┴" => (true, false, true, true).into(),

            // no east
            "├" => (true, true, false, true).into(),
            // no west
            "┤" => (true, true, true, false).into(),
            // all
            "┼" => (true, true, true, true).into(),

            _ => (false, false, false, false).into(),
        }
    }
}

impl From<Directions> for char {
    fn from(d: Directions) -> char {
        match (d.north, d.south, d.east, d.west) {
            // no turns
            (false, false, false, false) => ' ',
            // up-down
            (true, true, false, false) => '│',
            (true, false, false, false) => '╵',
            (false, true, false, false) => '╷',
            // left-right
            (false, false, true, true) => '─',
            (false, false, true, false) => '╴',
            (false, false, false, true) => '╶',
            // no north
            (false, true, true, false) => '┌',
            (false, true, false, true) => '┐',
            (false, true, true, true) => '┬',
            // no south
            (true, false, true, false) => '└',
            (true, false, false, true) => '┘',
            (true, false, true, true) => '┴',
            // no east
            (true, true, false, true) => '├',
            // no west
            (true, true, true, false) => '┤',
            // all
            (true, true, true, true) => '┼',
        }
    }
}
