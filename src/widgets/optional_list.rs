use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::Widget;
use std::fmt::{Debug, Display};

#[derive(Default, Debug, Clone)]
pub struct OptionalTextList<T: Display + Debug> {
    pub items: Vec<T>,
    lines: Vec<Line<'static>>,
    //lines: Vec<String>,
    // TODO
    // for now assuming everything can fit
    _offset: usize,
    selected: Option<usize>,
    hovered: Option<usize>,
    //selected: Option<usize>,
    //state: ListState,
    style: Style,
    selected_style: Style,
    hovered_style: Style,
}

impl<T: Display + Debug> OptionalTextList<T> {
    pub fn new(items: Vec<T>, style: Style, selected_style: Style,
        hovered_style: Style,
        ) -> OptionalTextList<T> {
        let mut lines: Vec<Line<'static>> = items
            .iter()
            .map(|i| Line::from(i.to_string()).style(style))
            .collect();
        if let Some(line) = lines.get_mut(0) {
            line.style = selected_style;
        }
        OptionalTextList {
            items,
            lines,
            //strings
            _offset: 0,
            selected: None,
            hovered: None,
            style,
            selected_style,
            hovered_style,
        }
    }

    pub fn default_style(items: Vec<T>) -> OptionalTextList<T> {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        OptionalTextList::new(items, style, selected_style, hovered_style)
    }

    pub fn select_previous(&mut self) {
        if let Some(selected) = self.selected {
            self.select(selected.saturating_sub(1));
        }
    }

    pub fn select_next(&mut self) {
        if let Some(selected) = self.selected {
            self.select(selected.saturating_add(1).min(self.items.len() - 1));
        }
    }

    pub fn select(&mut self, idx: usize) {
        self.hover(None);
        if let Some(selected) = self.selected {
            self.lines[selected].style = self.style;
        }
        let selected = idx.min(self.items.len() - 1);
        self.selected = Some(selected);
        self.lines[selected].style = self.selected_style;
    }

    pub fn hover(&mut self, idx: Option<usize>) {
        if let Some(prev_hovered) = self.hovered {
            self.lines[prev_hovered].style = self.style;
        }
        if let Some(idx) = idx
            && Some(idx) != self.selected
            && idx < self.items.len()
        {
            let hovered_idx = idx.min(self.items.len() - 1);
            self.hovered = Some(hovered_idx);
            self.lines[hovered_idx].style = self.hovered_style;
        } else {
            self.hovered = None;
        }
    }

    pub fn selected(&self) -> Option<&T> {
        if let Some(selected) = self.selected {
            Some(&self.items[selected])
        } else {
            None
        }
    }
}

impl<T: Display + Debug> Widget for OptionalTextList<T> {
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
