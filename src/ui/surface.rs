use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Style},
    widgets::block::title::Title,
    widgets::{Block, Borders, Gauge, List, ListState, Paragraph, Sparkline, Wrap},
};
use strum::IntoEnumIterator;

use crate::surface::tutorial::Tutorial;
use crate::theme::DEFAULT_STYLE;

use ratatui::prelude::*;

use crate::app::{App, InputMode};
use crate::surface::grid::Gent;
use crate::surface::{Focus, Surface};
use crate::ui::tech_tree::current_research_content;
use crate::ui::{render_stateful_widget_clamped, render_widget_clamped, render_widget_ref_clamped};

#[derive(Debug, Default)]
pub struct SurfaceLayout {
    pub surface: Rect,
    pub info: Rect,
    pub agent: AgentLayout,
    pub agents: Rect,
    pub power: PowerLayout,
    pub tutorial: TutorialLayout,
    pub tech: Rect,
    pub pause_menu_button: Rect,
    pub victory_popup: Rect,
}

impl SurfaceLayout {
    pub fn new(width: u16, height: u16) -> SurfaceLayout {
        let area = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Min(20)])
            .split(area);

        let left_col = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(chunks[0]);

        let right_col = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Fill(6),
                Constraint::Min(8),
                Constraint::Percentage(35),
            ])
            .split(chunks[1]);

        let tutorial = Rect {
            x: left_col[0].x,
            y: left_col[0].y,
            width: left_col[0].width,
            height: 5,
        };

        SurfaceLayout {
            surface: left_col[0],
            info: left_col[1],
            pause_menu_button: right_col[0],
            tech: right_col[1],
            agents: right_col[2],
            power: right_col[3].into(),
            //tutorial: right_col[3].into(),
            tutorial: tutorial.into(),
            agent: right_col[4].into(),
            victory_popup: area.inner(Margin::new(10, 10)),
        }
    }
}

#[derive(Debug, Default)]
pub struct AgentLayout {
    pub log: Rect,
    pub text_box: Option<Rect>,
}

impl From<Rect> for AgentLayout {
    fn from(rect: Rect) -> AgentLayout {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(3), Constraint::Max(2)])
            .split(rect.inner(Margin::new(1, 1)));
        AgentLayout {
            log: rect,
            text_box: Some(chunks[1]),
        }
    }
}

#[derive(Debug, Default)]
pub struct PowerLayout {
    pub area: Rect,
    pub spark_positive: Rect,
    pub spark_negative: Rect,
    pub accumulator: Rect,
}

impl From<Rect> for PowerLayout {
    fn from(rect: Rect) -> PowerLayout {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(2),
                Constraint::Max(2),
                Constraint::Max(2),
                Constraint::Max(1),
            ])
            .split(rect.inner(Margin::new(1, 1)));
        PowerLayout {
            area: rect,
            spark_positive: chunks[1],
            spark_negative: chunks[2],
            accumulator: chunks[3],
        }
    }
}

#[derive(Debug, Default)]
pub struct TutorialLayout {
    pub area: Rect,
    pub next_button: Rect,
    pub previous_button: Rect,
}

impl From<Rect> for TutorialLayout {
    fn from(rect: Rect) -> TutorialLayout {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(2), Constraint::Length(1)])
            .split(rect);
        let button_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(9),
                Constraint::Fill(2),
                Constraint::Length(9),
                Constraint::Length(1),
            ])
            .split(chunks[1]);
        TutorialLayout {
            area: rect,
            previous_button: button_row[1],
            next_button: button_row[3],
        }
    }
}

fn render_agent_log(app: &App, frame: &mut Frame) {
    let agent_layout = &app.layout.surface.agent;
    let height = agent_layout.log.height as usize;
    let width = agent_layout.log.width as usize;
    if let Some(comms) = app.surface.focused_agent_comms() {
        let agent_kind = comms.entity.line();
        let port = comms.port;
        match comms.address {
            Some(addr) => {
                let log_list = comms.log.list(height as u8, width);
                //let log_list = agent_log(comms, height, width);
                let widget = log_list
                    .block(
                        Block::default()
                            .title(agent_kind)
                            .title(port.to_string())
                            .title(Title::from(addr.to_string()).alignment(Alignment::Right))
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::Green).bg(Color::Black));
                render_widget_clamped(frame, widget, agent_layout.log);
            }
            None => {
                let style = Style::default().fg(Color::Red).bg(Color::Black);
                let b = Block::bordered()
                    .title(agent_kind)
                    .title(port.to_string())
                    .title(Title::from("disconnected").alignment(Alignment::Right))
                    .style(style);

                render_widget_clamped(frame, b, agent_layout.log);
                //let list = agent_log(comms, height, width).style(style);
                let list = comms.log.list(height as u8, width).style(style);

                if let Some(text_box_rect) = agent_layout.text_box {
                    let (color, append, shortcut) = match app.input_mode {
                        InputMode::Normal => (Color::Red, "", "[C]"),
                        InputMode::Editing => (Color::Green, "█", "[ESC]"),
                    };
                    let text_box =
                        Paragraph::new("> ".to_string() + comms.text_box.input.as_str() + append)
                            .block(
                                Block::default()
                                    .title("Command Line")
                                    .title(Title::from(shortcut).alignment(Alignment::Right))
                                    .borders(Borders::TOP)
                                    .fg(color)
                                    .bg(Color::Black),
                            );

                    render_widget_clamped(frame, list, agent_layout.log.inner(Margin::new(1, 1)));
                    render_widget_clamped(frame, ratatui::widgets::Clear, text_box_rect);
                    render_widget_clamped(frame, text_box, text_box_rect);
                }
            }
        }
    } else {
        let empty = List::new(Vec::<String>::new())
            .block(
                Block::default()
                    .title("non-agent selected")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Red).bg(Color::Black));
        render_widget_clamped(frame, empty, agent_layout.log);
    }
}

fn render_info(app: &App, frame: &mut Frame) {
    let buf = frame.buffer_mut();
    // TODO calculate center of screen?
    let surface_cords = format!("[{}, {}]", app.surface.x, app.surface.y);
    match app.surface.focus.clone() {
        Some(Focus::Position(pos)) => {
            // TODO use top left position of Large entities?
            if let Some(gent) = app.surface.grid.get(&pos) {
                let area = app.layout.surface.info;
                let inner = area.inner(Margin::new(1, 1));
                let cords = format!("[{}, {}]", pos.x, pos.y);
                Block::bordered()
                    .title(cords)
                    .title(gent.entity().line())
                    .title(Title::from(surface_cords).alignment(Alignment::Right))
                    .style(DEFAULT_STYLE)
                    .render(area, buf);
                render_widget_ref_clamped(frame, gent, inner);
            }
        }
        Some(Focus::Agent(port)) => {
            let area = app.layout.surface.info;
            if let Some(pos) = app.surface.agent_position(&port) {
                if let Some(gent) = app.surface.grid.get(&pos) {
                    //gent.render_ref(app.layout.surface.info, buf)
                    //render_widget_ref_clamped(frame, gent, app.layout.surface.info);
                    let inner = area.inner(Margin::new(1, 1));
                    let cords = format!("[{}, {}]", pos.x, pos.y);
                    Block::bordered()
                        .title(cords)
                        .title(gent.entity().line())
                        .title(Title::from(surface_cords).alignment(Alignment::Right))
                        .style(DEFAULT_STYLE)
                        .render(area, buf);
                    render_widget_ref_clamped(frame, gent, inner);
                }
            } else {
                Block::bordered()
                    .title("HUD")
                    .title(Title::from(surface_cords).alignment(Alignment::Right))
                    .style(DEFAULT_STYLE)
                    .render(area, buf);
            }
        }
        _ => Block::bordered()
            .style(DEFAULT_STYLE)
            .render(app.layout.surface.info, buf),
    }
}

fn render_agents_list(app: &App, frame: &mut Frame) {
    let mut agents: Vec<String> = vec![];
    for (port, comms) in app.surface.agents.iter() {
        if let Some(pos) = comms.position {
            if let Some(gent) = app.surface.grid.get(&pos) {
                agents.push(format!("{:?} {}", port, gent.entity()))
            }
        } else {
            agents.push(format!("{port:?} HUD"))
        }
    }
    let agents_ui = List::new(agents)
        .highlight_style(Style::new().bg(crate::theme::SELECTED_BG))
        // TODO Agents [A] navigation
        .block(Block::default().title("Agents").borders(Borders::ALL))
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    let mut state = ListState::default();
    match app.surface.focus {
        Some(Focus::Agent(address)) => {
            let adrs: Vec<usize> = app.surface.agents.keys().cloned().collect();
            if let Some(idx) = adrs.iter().position(|&a| a == address) {
                state.select(Some(idx));
            }
        }
        _ => state.select(None),
    }
    render_stateful_widget_clamped(frame, agents_ui, app.layout.surface.agents, &mut state);
}

fn render_surface_grid(app: &App, frame: &mut Frame) {
    let area = app.layout.surface.surface;
    app.surface.render_grid(frame, area);
    let _buf = frame.buffer_mut();
}

fn render_surface_grid_fx(app: &mut App, frame: &mut Frame) {
    let area = app.layout.surface.surface;
    app.surface.render_agent_fx(frame, area, app.prev_tick);
}

fn render_surface_overlay(app: &App, frame: &mut Frame) {
    let buf = frame.buffer_mut();
    match &app.surface.focus {
        Some(Focus::Agent(port)) => {
            if *port == 3333 {
                return;
            }
            let grid_pos = app.surface.agent_position(port);
            let agent = app.surface.get_agent(port);
            match (grid_pos, agent) {
                (Some(grid_pos), Some(agent)) => {
                    highlight(&grid_pos, agent.entity().footprint(), &app.surface, buf);
                }
                _ => tracing::error!("expected agent at port {port}"),
            }
        }
        Some(Focus::Position(grid_pos)) => match app.surface.grid.get_direct(grid_pos) {
            Some(Gent::Intmd(p)) => highlight(grid_pos, p.footprint(), &app.surface, buf),
            Some(Gent::Large(pos)) => {
                if let Some(Gent::Intmd(entity)) = app.surface.grid.get(pos) {
                    highlight(pos, entity.footprint(), &app.surface, buf)
                }
            }
            Some(Gent::Empty) => highlight(grid_pos, None, &app.surface, buf),
            _ => (),
        },
        //        Some(Focus::Position(grid_pos)) => match app.surface.grid.get(grid_pos) {
        //            Some(Gent::Intmd(p)) => {
        //                highlight(&grid_pos, p.footprint(), &app.surface, buf)
        //            }
        //            Some(Gent::Empty) => highlight(&grid_pos, None, &app.surface, buf),
        //            _ => (),
        //        },
        _ => (),
    }
    if let Some(stats) = &app.surface.victory_stats
        && stats.show_victory
    {
        Paragraph::new(stats.stats.to_string())
            .centered()
            .block(Block::bordered().title("!!! VICTORY !!!"))
            .style(DEFAULT_STYLE)
            .render(app.layout.surface.victory_popup, buf);
    }
}

fn highlight(
    grid_pos: &Position,
    footprint: Option<Position>,
    surface: &Surface,
    buf: &mut Buffer,
) {
    let fp = footprint.unwrap_or(Position { x: 1, y: 1 });
    let rect = Rect {
        x: grid_pos.x,
        y: grid_pos.y,
        width: fp.x,
        height: fp.y,
    };
    for grid_pos in rect.positions() {
        if let Some(frame_pos) = surface.frame_position(&grid_pos)
            && let Some(cell) = buf.cell_mut(frame_pos)
        {
            cell.set_bg(crate::theme::SELECTED_BG);
        }
    }
}

// TODO performance
// since this is a hot loop:
// * move {generation,consumption}_history to Power
fn render_power(app: &App, frame: &mut Frame) {
    let power = &app.surface.power;

    let net_power = power.generation as i32 - power.consumption as i32;
    let old = Paragraph::new(format!(
        "+ {}j - {}j = {}j",
        power.generation, power.consumption, net_power
    ))
    .alignment(Alignment::Center)
    .block(Block::bordered().title("Power").style(DEFAULT_STYLE))
    .style(Style::default().fg(Color::Green).bg(Color::Black));
    let percent =
        ((f64::from(power.stored as u32) / f64::from(power.capacity as u32)) * 100.0) as u16;
    let label = format!("{}Kj/{}Kj", power.stored / 1000, power.capacity / 1000);
    let storage = Gauge::default()
        .gauge_style(
            Style::default()
                .bg(Color::Black)
                .fg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        )
        .label(&label)
        .percent(percent);
    let generation_max = power.generation_history.iter().max().unwrap_or(&0);
    let consumption_max = power.consumption_history.iter().max().unwrap_or(&0);
    let max = generation_max.max(consumption_max);
    let net_history: Vec<isize> = power
        .generation_history
        .iter()
        .zip(power.consumption_history.iter())
        .map(|(&g, &c)| g as isize - c as isize)
        .collect();
    let generation_history: Vec<u64> = net_history.iter().map(|n| *n.max(&0) as u64).collect();
    let consumption_history: Vec<u64> = net_history
        .iter()
        .map(|n| {
            if *n >= 0 {
                *max
            } else {
                (*max as isize + n) as u64
            }
        })
        .collect();

    let spark = Sparkline::default()
        .data(&generation_history)
        .style(Style::default().green().on_black())
        .max(*max);
    let spark2 = Sparkline::default()
        .data(&consumption_history)
        .style(Style::default().black().on_red())
        .max(*max);

    render_widget_clamped(frame, old, app.layout.surface.power.area);
    render_widget_clamped(frame, spark, app.layout.surface.power.spark_positive);
    render_widget_clamped(frame, spark2, app.layout.surface.power.spark_negative);
    render_widget_clamped(frame, storage, app.layout.surface.power.accumulator);
}

fn render_tutorial(app: &App, frame: &mut Frame) {
    // better way to clear?
    let buf = &mut frame.buffer_mut();
    for pos in app.layout.surface.tutorial.area.positions() {
        buf[(pos.x, pos.y)].reset();
    }

    let tutorial_state = &app.surface.game_state.tutorial_state;

    let old = Paragraph::new(tutorial_state.instructions())
        .wrap(Wrap { trim: false })
        .block(Block::bordered().title("Tutorial").title(format!(
            "{}/{}",
            tutorial_state.progress(),
            Tutorial::iter().count() - 2
        )))
        .style(DEFAULT_STYLE);

    render_widget_clamped(frame, old, app.layout.surface.tutorial.area);
    render_widget_clamped(
        frame,
        app.tutorial_previous_button,
        app.layout.surface.tutorial.previous_button,
    );
    render_widget_clamped(
        frame,
        app.tutorial_next_button,
        app.layout.surface.tutorial.next_button,
    );
}

pub fn render(app: &mut App, frame: &mut Frame) {
    render_surface_grid(app, frame);
    render_surface_overlay(app, frame);
    //render_info(app, frame);
    render_info(app, frame);
    render_agent_log(app, frame);
    render_agents_list(app, frame);
    render_power(app, frame);
    if app.surface.game_state.tutorial_state != Tutorial::Complete {
        render_tutorial(app, frame)
    }

    let (name, guage) = current_research_content(app);
    let titles = [
        Some(format!("Researching: {name}")),
        None,
        Some("[T]".to_string()),
    ];
    let tech = app
        .current_research_button
        .with_content_and_title(guage, titles);
    render_widget_clamped(frame, &tech, app.layout.surface.tech);

    render_widget_clamped(
        frame,
        &app.pause_menu_button,
        app.layout.surface.pause_menu_button,
    );
    render_surface_grid_fx(app, frame);
}
