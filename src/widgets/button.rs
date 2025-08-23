use std::marker::PhantomData;

use crate::widgets::HandleInput;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Margin, Rect},
    prelude::Position,
    style::{Color, Style, Styled},
    text::Span,
    widgets::{Block, BorderType, WidgetRef, block::Title},
};

// TODO make this str
pub type Titles = [Option<String>; 3];

#[derive(Debug, Clone)]
pub struct Button<K, W: WidgetRef + Styled<Item = W>> {
    pub content: W,
    border: Option<Block<'static>>,
    style: Style,
    default_style: Style,
    // if we want/need a toggle style button or want to handle MouseEvent::{Up,Down} "properly"
    // pressed_style: Style,
    hovered_style: Style,
    // for polymorphism over impl WidgetRef
    kind: PhantomData<K>,
}

impl<K, W: Clone + WidgetRef + Styled<Item = W>> HandleInput for Button<K, W> {
    type Output = ();
    fn handle_mouse_event(&mut self, event: MouseEvent, _rel_pos: Position) -> Option<()> {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => Some(()),
            MouseEventKind::Moved => {
                self.hovered(true);
                None
            }
            _ => None,
        }
    }

    fn on_mouse_elsewhere(&mut self) {
        self.hovered(false);
    }
}

impl<K, W: Clone + WidgetRef + Styled<Item = W>> Button<K, W> {
    pub fn new<T: Into<W>>(content: T) -> Self {
        Button {
            content: content.into(),
            border: None,
            style: Style::new().bg(Color::Black).fg(Color::Green),
            default_style: Style::new().bg(Color::Black).fg(Color::Green),
            hovered_style: Style::new().bg(Color::Black).fg(Color::LightGreen),
            kind: PhantomData,
        }
    }

    pub fn hovered(&mut self, hovered: bool) {
        if hovered {
            self.style = self.hovered_style;
        } else {
            self.style = self.default_style;
        }
    }

    pub fn set_content(&mut self, content: W) {
        self.content = content;
    }

    pub fn set_titles(&mut self, titles: [Option<String>; 3]) {
        self.border = Some(into_block(titles));
    }

    pub fn with_default_border(self) -> Self {
        Button {
            border: Some(
                Block::bordered()
                    .border_type(BorderType::Thick)
                    .border_style(Style::default().bg(Color::Black).fg(Color::Green)),
            ),
            ..self
        }
    }

    pub fn with_content(self, content: W) -> Self {
        Button {
            content,
            border: self.border.clone(),
            ..self
        }
    }

    pub fn with_titles(self, titles: [Option<String>; 3]) -> Self {
        Button {
            border: Some(into_block(titles)),
            ..self
        }
    }
}

fn into_block(titles: Titles) -> Block<'static> {
    let mut block = Block::bordered()
        .border_type(BorderType::Thick)
        .border_style(Style::default().bg(Color::Black).fg(Color::Green));
    let [l, c, r] = titles;
    if let Some(title) = l {
        block = block.title(Title::from(title).alignment(Alignment::Left))
    }
    if let Some(title) = c {
        block = block.title(Title::from(title).alignment(Alignment::Center))
    }
    if let Some(title) = r {
        block = block.title(Title::from(title).alignment(Alignment::Right))
    }
    block
}

#[derive(Default, Debug, Copy, Clone)]
pub struct TextButtonType;
pub type TextButton<'a> = Button<TextButtonType, Span<'a>>;

impl WidgetRef for TextButton<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        self.content.render_ref(area, buf);
        buf.set_style(area, self.style);
    }
}

#[derive(Default, Debug, Copy, Clone)]
pub struct BorderedButtonType;
pub type BorderedButton<T> = Button<BorderedButtonType, T>;

impl<W: WidgetRef + Styled<Item = W>> WidgetRef for BorderedButton<W> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        if let Some(border) = &self.border {
            border.render_ref(area, buf);
        }
        let inner = area.inner(Margin::new(1, 1));
        self.content.render_ref(inner, buf);
        buf.set_style(area, self.style);
    }
}
