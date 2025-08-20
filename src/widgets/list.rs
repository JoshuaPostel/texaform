use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Widget, WidgetRef};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use crate::widgets::{HandleInput, DoubleClickTracker};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

#[derive(Default, Debug, Clone)]
pub struct AlignedLine {
    left: Option<String>,
    center: Option<String>,
    right: Option<String>,
    style: Style,
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

    pub fn style(self, style: Style) -> AlignedLine {
        AlignedLine { style, ..self }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TextList<I, T: Display + Debug> {
    items: Vec<T>,
    lines: Vec<AlignedLine>,
    // TODO
    // for now assuming everything can fit
    _offset: usize,
    selected: Option<usize>,
    hovered: Option<usize>,
    style: Style,
    selected_style: Style,
    hovered_style: Style,
    double_click_tracker: DoubleClickTracker<usize>,
    // for polymorphism over HandleInput
    handle_input_kind: PhantomData<I>
}

#[derive(Default, Debug, Copy, Clone)]
pub struct DoubleClickListType;
pub type DoubleClickList<T> = TextList<DoubleClickListType, T>;

pub enum Action<T> {
    // TODO better names
    Select(T),
    Choose(T),
}

impl<T: Display + Debug + Clone> HandleInput for DoubleClickList<T> {
    // TODO try returning reference to T
    type Output = Action<T>;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Self::Output> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                self.selected().map(|i| Action::Select(i.clone()))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                self.selected().map(|i| Action::Select(i.clone()))
            }
            KeyCode::Enter => self.selected().map(|i| Action::Choose(i.clone())),
            _ => None,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: MouseEvent,
        relative_position: Position,
    ) -> Option<Self::Output> {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // TODO test/think through clicking beyond list length
                let idx = relative_position.y as usize;
                self.select(idx);
                if self.double_click_tracker.clicked(idx) {
                    self.selected().map(|i| Action::Choose(i.clone()))
                } else {
                    self.selected().map(|i| Action::Select(i.clone()))
                }
            }
            MouseEventKind::Moved => {
                self.hover(Some(relative_position.y as usize));
                None
            }
            _ => None,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct ClickListType;
pub type ClickList<T> = TextList<ClickListType, T>;

impl<T: Display + Debug + Clone> HandleInput for ClickList<T> {
    // TODO try returning reference to T
    type Output = T;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Self::Output> {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                None
            }
            KeyCode::Enter => self.selected().map(|i| i.clone()),
            _ => None,
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: MouseEvent,
        relative_position: Position,
    ) -> Option<Self::Output> {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.select(relative_position.y as usize);
                self.selected().map(|i| i.clone())
            }
            MouseEventKind::Moved => {
                self.select(relative_position.y as usize);
                None
            }
            _ => None,
        }
    }
}

impl<I, T: Display + Debug + Ord + Eq> TextList<I, T> {
    // TODO need to have T impl Into<AlignedLine>
    // otherwise rebuild lines may create different lines than supplied by
    // pub fn default_style_with_lines
    fn rebuild_lines(&mut self) {
        self.lines = self
            .items
            .iter()
            .map(|i| AlignedLine::from(i.to_string()).style(self.style))
            .collect();
        if let Some(idx) = self.selected {
            self.lines[idx].style = self.style;
        }
        if let Some(idx) = self.hovered {
            self.lines[idx].style = self.hovered_style;
        }
    }

    pub fn sort(&mut self) {
        self.items.sort();
        self.rebuild_lines();
    }

    pub fn insert(&mut self, item: T) {
        if !self.items.contains(&item) {
            self.items.push(item);
            self.sort();
        }
    }

    /// returns whether or not item was in list
    pub fn select_item(&mut self, item: &T) -> bool {
        if let Some(idx) = self.items.iter().position(|i| i == item) {
            self.select(idx);
            true
        } else {
            false
        }
    }
}

impl<I, T: Display + Debug> TextList<I, T> {
    fn new(
        items: Vec<T>,
        style: Style,
        selected_style: Style,
        hovered_style: Style,
    ) -> Self {
        let lines = items
            .iter()
            .map(|i| AlignedLine::from(i.to_string()).style(style))
            .collect();
        TextList {
            items,
            lines,
            _offset: 0,
            selected: None,
            hovered: None,
            style,
            selected_style,
            hovered_style,
            double_click_tracker: DoubleClickTracker::default(),
            handle_input_kind: PhantomData,
        }
    }

    pub fn default_style(items: Vec<T>) -> Self {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        TextList::new(items, style, selected_style, hovered_style)
    }

    pub fn default_style_with_lines(items: Vec<T>, lines: Vec<AlignedLine>) -> Self {
        let style = Style::new().fg(Color::Green).bg(Color::Black);
        let selected_style = Style::new().fg(Color::Black).bg(Color::Green);
        let hovered_style = Style::new().fg(Color::Black).bg(Color::DarkGray);
        TextList {
            items,
            lines,
            _offset: 0,
            selected: None,
            hovered: None,
            style,
            selected_style,
            hovered_style,
            double_click_tracker: DoubleClickTracker::default(),
            handle_input_kind: PhantomData,
        }
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

    // TODO will this panic if called on empty list?
    pub fn select(&mut self, idx: usize) {
        self.hover(None);
        if let Some(selected) = self.selected {
            self.lines[selected].style = self.style;
        }
        let selected = idx.min(self.items.len().saturating_sub(1));
        self.selected = Some(selected);
        self.lines[selected].style = self.selected_style;
    }

    /// returns whether or not item was in list
    pub fn select_by_display(&mut self, item: &impl Display) -> bool {
        if let Some(idx) = self
            .items
            .iter()
            .position(|i| i.to_string() == item.to_string())
        {
            self.select(idx);
            true
        } else {
            false
        }
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

    pub fn selected_unchecked(&self) -> &T {
        &self.items[self.selected.unwrap()]
    }
}

impl<I, T: Display + Debug> Widget for TextList<I, T> {
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
