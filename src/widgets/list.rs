use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Position, Rect};
use ratatui::style::{Color, Style, Styled, Stylize};
use ratatui::text::{Line, Span, ToSpan};
use ratatui::widgets::{Widget, WidgetRef};
use std::fmt::{Debug, Display};

#[derive(Default, Debug, Clone)]
pub struct AlignedLine {
    left: Option<String>,
    center: Option<String>,
    right: Option<String>,
    style: Style,
}

impl Widget for AlignedLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(left) = self.left {
            Line::from(left)
                .alignment(Alignment::Left)
                .render(area, buf);
        }
        if let Some(center) = self.center {
            Line::from(center)
                .alignment(Alignment::Center)
                .render(area, buf);
        }
        if let Some(right) = self.right {
            Line::from(right)
                .alignment(Alignment::Right)
                .render(area, buf);
        }
        buf.set_style(area, self.style);
    }
}

impl WidgetRef for AlignedLine {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if let Some(left) = &self.left {
            Line::from(left.as_str())
                .alignment(Alignment::Left)
                .render_ref(area, buf);
        }
        if let Some(center) = &self.center {
            Line::from(center.as_str())
                .alignment(Alignment::Center)
                .render_ref(area, buf);
        }
        if let Some(right) = &self.right {
            Line::from(right.as_str())
                .alignment(Alignment::Right)
                .render_ref(area, buf);
        }
        buf.set_style(area, self.style);
    }
}

impl From<String> for AlignedLine {
    fn from(s: String) -> AlignedLine {
        AlignedLine {
            left: Some(s),
            ..Default::default()
        }
    }
}

impl AlignedLine {
    pub fn left_right(l: String, r: String) -> AlignedLine {
        AlignedLine {
            left: Some(l),
            right: Some(r),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextList<T: Display + Debug> {
    pub items: Vec<T>,
    lines: Vec<AlignedLine>,
    // TODO
    // for now assuming everything can fit
    _offset: usize,
    selected: usize,
    // TODO
    //selected: Option<usize>,
    hovered: Option<usize>,
    style: Style,
    selected_style: Style,
    hovered_style: Style,
}

impl<T: Display + Debug> TextList<T> {
    pub fn default_style(items: Vec<T>) -> TextList<T> {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        let lines = items
            .iter()
            .map(|i| AlignedLine::from(i.to_string()))
            .collect();
        let mut test_list = TextList {
            items,
            lines,
            _offset: 0,
            selected: 0,
            hovered: None,
            style,
            selected_style,
            hovered_style,
        };
        test_list.lines[0].style = selected_style;
        test_list
    }

    pub fn default_style_with_lines(items: Vec<T>, lines: Vec<AlignedLine>) -> TextList<T> {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        let mut test_list = TextList {
            items,
            lines,
            _offset: 0,
            selected: 0,
            hovered: None,
            style,
            selected_style,
            hovered_style,
        };
        test_list.lines[0].style = selected_style;
        test_list
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
            // TODO AlignedLine with owned string instead?
            //line.render(line_area, buf);
            line.clone().render(line_area, buf);
        }
    }
}
