pub mod rectangle;

use ratatui::prelude::*;
use ratatui::style::{Color, Modifier};

// because https://docs.rs/ratatui/latest/ratatui/buffer/struct.Cell.html cant be constructed
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct PubCell {
    pub c: char,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl PubCell {
    pub fn from_char(c: char) -> PubCell {
        PubCell {
            c,
            bg: Color::Black,
            ..Default::default()
        }
    }

    pub fn set(self, cell: &mut ratatui::buffer::Cell) {
        cell.set_char(self.c);
        cell.set_fg(self.fg);
        cell.set_bg(self.bg);
        cell.set_style(Style::default().add_modifier(self.modifier));
    }
}

impl std::cmp::PartialEq<char> for PubCell {
    fn eq(&self, other: &char) -> bool {
        &self.c == other
    }
}

impl From<char> for PubCell {
    fn from(c: char) -> PubCell {
        PubCell::from_char(c)
    }
}

pub trait SetCell {
    fn set_cell(&mut self, x: u16, y: u16, pub_cell: PubCell);
}

impl SetCell for ratatui::Frame<'_> {
    fn set_cell(&mut self, x: u16, y: u16, pub_cell: PubCell) {
        let buf = self.buffer_mut();
        let cell = &mut buf[(x, y)];
        pub_cell.set(cell);
    }
}

impl SetCell for ratatui::buffer::Buffer {
    fn set_cell(&mut self, x: u16, y: u16, pub_cell: PubCell) {
        let cell = &mut self[(x, y)];
        pub_cell.set(cell);
    }
}
