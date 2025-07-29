use crossterm::event::MouseEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;
use std::fmt::{Debug, Display};

#[derive(Debug, Clone)]
pub struct TextList<T: Display + Debug> {
    pub items: Vec<T>,
    lines: Vec<Line<'static>>,
    //lines: Vec<String>,
    // TODO
    // for now assuming everything can fit
    _offset: usize,
    selected: usize,
    hovered: Option<usize>,
    //selected: Option<usize>,
    //state: ListState,
    style: Style,
    selected_style: Style,
    hovered_style: Style,
}

impl<T: Display + Debug> TextList<T> {
    pub fn new(
        items: Vec<T>,
        style: Style,
        selected_style: Style,
        hovered_style: Style,
    ) -> TextList<T> {
        let mut lines: Vec<Line<'static>> = items
            .iter()
            .map(|i| Line::from(i.to_string()).style(style))
            .collect();
        lines[0].style = selected_style;
        TextList {
            items,
            lines,
            //strings
            _offset: 0,
            selected: 0,
            hovered: None,
            style,
            selected_style,
            hovered_style,
        }
    }

    pub fn default_style(items: Vec<T>) -> TextList<T> {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        TextList::new(items, style, selected_style, hovered_style)
    }

    pub fn select_previous(&mut self) {
        self.select(self.selected.saturating_sub(1));
    }

    pub fn select_next(&mut self) {
        self.select(self.selected.saturating_add(1).min(self.items.len() - 1));
    }

    pub fn select(&mut self, idx: usize) {
        tracing::info!("{idx}");
        tracing::info!("before: {}", self.selected);
        self.hover(None);
        self.lines[self.selected].style = self.style;
        self.selected = idx.min(self.items.len() - 1);
        self.lines[self.selected].style = self.selected_style;
        tracing::info!("after: {}", self.selected);
    }

    pub fn hover(&mut self, idx: Option<usize>) {
        if let Some(prev_hovered) = self.hovered {
            self.lines[prev_hovered].style = self.style;
        }
        if let Some(idx) = idx
            && idx != self.selected
            && idx < self.items.len()
        {
            let hovered_idx = idx.min(self.items.len() - 1);
            self.hovered = Some(hovered_idx);
            self.lines[hovered_idx].style = self.hovered_style;
        } else {
            self.hovered = None;
        }
    }

    pub fn selected(&self) -> &T {
        &self.items[self.selected]
    }
}

impl<T: Display + Debug> Widget for TextList<T> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (idx, line) in self.lines.iter().enumerate() {
            // TODO bounds checks (i.e. rendering list longer than area)
            let y = area.y + idx as u16;
            let line_area = Rect {
                x: area.x,
                y,
                width: area.width,
                height: 1,
            };
            line.render(line_area, buf);
        }
    }
}
