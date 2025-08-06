use ratatui::Frame;
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use serde::{Deserialize, Serialize};
use strum::EnumMessage;

use crate::agents::Agent;
use crate::entities::{Entity, PickResult};

use crate::utils::checked_pos_to_idx;
use crate::utils::pos_to_idx;

use ratatui::buffer::Buffer;
use ratatui::widgets::{Paragraph, WidgetRef};

use super::GRID_SIZE;

#[derive(Debug, Serialize, Deserialize)]
pub enum Gent {
    Empty,
    BeingUpdated,
    Intmd(Entity),
    Age(Box<dyn Agent>),
    Large(Position),
}

impl Gent {
    pub fn is_empty(&self) -> bool {
        matches!(self, Gent::Empty)
    }
}

impl WidgetRef for Gent {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match self {
            Gent::Intmd(e) => Paragraph::new(e.get_message().expect("all entities have message"))
                .render_ref(area, buf),
            Gent::Age(a) => a.render_ref(area, buf),
            Gent::Empty => Paragraph::new("null and void").render_ref(area, buf),
            _ => (),
        }
    }
}

impl Gent {
    pub fn entity(&self) -> Entity {
        match self {
            Gent::Intmd(p) => *p,
            Gent::Age(a) => a.entity(),
            _ => Entity::Empty,
        }
    }
    // used in ui/load_game.rs
    // just implement there?
    // render pointing at the inital starting position via GRID_SIZE
    pub fn render_surface_cell(&self, pos: &Position, cell: &mut Cell) {
        match self {
            Gent::Intmd(p) => {
                cell.set_char(p.character());
                cell.fg = p.fg();
                cell.bg = p.bg();
            }
            Gent::Age(a) => a.render_surface_cell(pos, cell),
            _ => (),
        }
    }

    pub fn pick(&mut self, c: char) -> PickResult {
        match self {
            Gent::Empty => PickResult::noop(),
            Gent::Age(a) => a.pick(c),
            Gent::Large(_) => PickResult::noop(),
            Gent::Intmd(p) => {
                if c == p.character() {
                    PickResult {
                        picked: Some(*p),
                        replace: Some(Gent::Empty),
                    }
                } else {
                    PickResult::noop()
                }
            }
            Gent::BeingUpdated => todo!(),
        }
    }

    pub fn placable(&self, entity: &Entity) -> bool {
        match self {
            Gent::Empty => false,
            Gent::Intmd(_) => false,
            Gent::Age(a) => a.placable(entity),
            Gent::Large(_) => false,
            Gent::BeingUpdated => todo!(),
        }
    }

    pub fn place(&mut self, entity: Entity) {
        match self {
            Gent::Empty => {
                *self = Gent::Intmd(entity);
            }
            Gent::Intmd(_) => unreachable!("cannot place into intermediate"),
            Gent::Age(a) => a.place(entity),
            Gent::Large(_) => todo!(),
            Gent::BeingUpdated => todo!(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Grid {
    raw: Vec<Gent>,
}

impl Grid {
    pub fn new(raw: Vec<Gent>) -> Grid {
        Grid { raw }
    }

    pub fn get(&self, pos: &Position) -> Option<&Gent> {
        let idx = pos_to_idx(pos, GRID_SIZE);
        match self.raw.get(idx) {
            Some(Gent::Large(pos)) => self.get(pos),
            gent => gent,
        }
    }
    pub fn get_mut(&mut self, pos: &Position) -> Option<&mut Gent> {
        let idx = pos_to_idx(pos, GRID_SIZE);
        match self.raw.get(idx) {
            Some(Gent::Large(pos)) => {
                let idx = pos_to_idx(pos, GRID_SIZE);
                self.raw.get_mut(idx)
            }
            _ => self.raw.get_mut(idx),
        }
    }
    pub fn pop(&mut self, pos: &Position) -> Option<Gent> {
        let idx = pos_to_idx(pos, GRID_SIZE);
        let len = self.raw.len();
        //self.raw.push(Gent::Empty);
        self.raw.push(Gent::BeingUpdated);
        self.raw.swap(idx, len);
        self.raw.pop()
    }
    pub fn insert(&mut self, pos: &Position, gent: Gent) {
        let idx = pos_to_idx(pos, GRID_SIZE);
        self.raw[idx] = gent;
    }
    pub fn get_direct(&self, pos: &Position) -> Option<&Gent> {
        //        let idx = checked_pos_to_idx(pos, GRID_SIZE);
        //        self.raw.get(idx)
        if let Some(idx) = checked_pos_to_idx(pos, GRID_SIZE) {
            self.raw.get(idx)
        } else {
            None
        }
    }
    pub fn pick(&mut self, c: char, pos: &Position) -> Option<Entity> {
        //if let Some(mut gent) = self.pop(pos) {
        if let Some(gent) = self.get_mut(pos) {
            let pick_result = gent.pick(c);
            if let Some(replace) = pick_result.replace {
                self.insert(pos, replace);
            }
            pick_result.picked
        } else {
            None
        }
    }
    pub fn buildable(&self, rect: Rect) -> bool {
        rect.positions()
            .all(|pos| self.get(&pos).map(|g| g.is_empty()).unwrap_or(false))
    }
    pub fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let no_offset = Position { x: 0, y: 0 };
        let clamped_area = area.clamp(frame.area());
        let buf = frame.buffer_mut();
        for pos in clamped_area.positions() {
            let cell = &mut buf[(pos.x, pos.y)];
            match self.get_direct(&pos) {
                None => {
                    cell.set_char('O');
                }
                Some(Gent::Intmd(p)) => {
                    cell.set_char(p.character());
                    cell.bg = Color::Black;
                }
                Some(Gent::Age(agent)) => {
                    agent.render_surface_cell(&no_offset, cell);
                }
                Some(Gent::Large(ent_pos)) => {
                    if let Some(Gent::Age(agent)) = self.get_direct(ent_pos) {
                        let offset = Position {
                            x: pos.x - ent_pos.x,
                            y: pos.y - ent_pos.y,
                        };
                        agent.render_surface_cell(&offset, cell);
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
}
