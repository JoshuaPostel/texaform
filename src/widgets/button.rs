use ratatui::{
    buffer::Buffer,
    layout::{Margin, Rect},
    style::{Color, Style, Styled},
    widgets::{Block, BorderType, Paragraph, Widget},
    text::Line,
};

#[derive(Debug, Clone)]
pub struct Button<W: Widget + Styled> {
    pub content: W,
    // TODO how to do a title while retaining Copy?
    pub title: Option<String>,
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub style: Style,
    pub pressed_style: Style,
    pub hovered_style: Style,
}

impl<W: Widget + Styled> Button<W> {
    pub fn new(content: W) -> Button<W> {
        Button {
            content,
            title: None,
            is_pressed: false,
            is_hovered: false,
            style: Style::new().bg(Color::Black).fg(Color::Green),
            pressed_style: Style::new().bg(Color::Black).fg(Color::Red),
            hovered_style: Style::new().bg(Color::Black).fg(Color::LightGreen),
        }
    }

    pub fn with_content(&self, content: W) -> Button<W> {
        Button {
            content,
            title: self.title.clone(),
            ..*self
        }
    }

    pub fn with_content_and_title(&self, content: W, title: String) -> Button<W> {
        Button {
            title: Some(title),
            content,
            ..*self
        }
    }
}

impl<W: Widget + Styled> Widget for Button<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = match (self.is_pressed, self.is_hovered) {
            (true, _) => self.pressed_style,
            (false, true) => self.hovered_style,
            _ => self.style,
        };
        // TODO PERF move to struct itself
        let mut block = Block::bordered()
            .border_type(BorderType::Thick)
            .border_style(Style::default().bg(Color::Black))
            .style(style);
        if let Some(title) = self.title {
            block = block.title(title);
        }
        block.render(area, buf);
        let inner = area.inner(Margin::new(1, 1));
        // TODO why not like this? open a discussion?
        //let foo = self.content.set_style(style);
        //foo.render(inner, buf);
        self.content.render(inner, buf);
        buf.set_style(inner, style)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextButton {
    pub content: &'static str,
    pub is_pressed: bool,
    pub is_hovered: bool,
    pub style: Style,
    pub pressed_style: Style,
    pub hovered_style: Style,
}

impl TextButton {
    pub fn new(content: &'static str) -> TextButton {
        TextButton {
            content,
            is_pressed: false,
            is_hovered: false,
            style: Style::new().bg(Color::Black).fg(Color::Green),
            pressed_style: Style::new().bg(Color::Black).fg(Color::Red),
            hovered_style: Style::new().bg(Color::Black).fg(Color::LightGreen),
        }
    }

    pub fn width(&self) -> u16 {
        self.content.len() as u16
    }
}

impl Widget for TextButton {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = match (self.is_pressed, self.is_hovered) {
            (true, _) => self.pressed_style,
            (false, true) => self.hovered_style,
            _ => self.style,
        };
        Line::from(self.content) 
            .style(style).render(area, buf);
    }
}

#[derive(Clone)]
pub enum Location {
    East(i16),
    SouthEast,
    // implement the rest as needed
}

#[derive(Clone)]
pub struct BorderAttachedButton {
    pub button: Button<Paragraph<'static>>,
    attached_direction: Location,
    n_chars: u16,
}

impl BorderAttachedButton {
    pub fn update(&mut self, text: String) {
        self.n_chars = text.len() as u16;
        self.button.content = Paragraph::new(text).centered();
    }

    pub fn resize(&self, width: u16, height: u16) -> Rect {
        match self.attached_direction {
            Location::SouthEast => {
                Rect {
                    x: width - (self.n_chars + 2),
                    y: height - 3,
                    width: self.n_chars + 2,
                    height: 3,
                }
            },
            Location::East(i) => {
                Rect {
                    x: width - (self.n_chars + 2),
                    y: ((height as i16) - i) as u16,
                    width: self.n_chars + 2,
                    height: 3,
                }
            },
        }
    }

    pub fn new(text: String, attached_direction: Location) -> BorderAttachedButton {
        let n_chars = text.len() as u16;
        let content = Paragraph::new(text);

        let button = Button {
            content,
            title: None,
            is_pressed: false,
            is_hovered: false,
            style: Style::new().bg(Color::Black).fg(Color::Green),
            pressed_style: Style::new().bg(Color::Black).fg(Color::Red),
            hovered_style: Style::new().bg(Color::Black).fg(Color::LightGreen),
        };
        BorderAttachedButton { button,attached_direction, n_chars }
    }
}

impl Widget for BorderAttachedButton {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.button.render(area, buf);
        match self.attached_direction {
            Location::East(_) => {
                let cell = &mut buf[(area.right() - 1, area.top())];
                cell.set_char('┪');
                let cell = &mut buf[(area.right() - 1, area.bottom() - 1)];
                cell.set_char('┩');
            },
            Location::SouthEast => {
                let cell = &mut buf[(area.right() - 1, area.top())];
                cell.set_char('┪');
                let cell = &mut buf[(area.left(), area.bottom() - 1)];
                cell.set_char('┺');
            },
        }
        // Not sure why adding whitespace to button.content does not do this for us
        // possibly a ratatui optimization leading to a "bug" or unexpected behavior?
        let cell = &mut buf[(area.left() + 1, area.bottom() - 2)];
        cell.set_char(' ');
        let cell = &mut buf[(area.right() - 2, area.top() + 1)];
        cell.set_char(' ');
    }
}
