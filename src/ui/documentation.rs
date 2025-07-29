use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph, Wrap},
};
use strum::VariantArray;
use strum_macros;

use crate::agents::{self, dog, fabricator, hud, laser_cutter, smelter};
use crate::app::App;

use crate::effects::Effects;
use crate::entities;
use crate::ui::{AppLayout, render_effect_clamped, render_widget_clamped};

use std::time::Duration;

#[derive(Debug, Default)]
pub struct DocumentationLayout {
    pub list: Rect,
    pub document: Rect,
    pub copy_button: Rect
}

impl DocumentationLayout {
    pub fn new(width: u16, height: u16, app: &App) -> DocumentationLayout {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(15), Constraint::Fill(1)])
            .split(Rect {
                x: 0,
                y: 0,
                width,
                height,
            });

        let copy_button = app.copy_button.resize(width, height);
//        let copy_button = rect { //x: width - (self.n_chars + 2),
//            x: width - 21,
//            y: height - 6,
//            //width: self.n_chars + 2,
//            width: 21,
//            height: 3,
//        };

        DocumentationLayout {
            list: chunks[0],
            document: chunks[1],
            copy_button,
            // TODO move documentataion_scroll here?
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let border = Block::bordered()
        .title("Documentation")
        .style(Style::new().bg(Color::Black).fg(Color::Green));
    render_widget_clamped(frame, border, app.layout.documentation.list);
    render_widget_clamped(
        frame,
        app.documentation.clone(),
        app.layout.documentation.list.inner(Margin::new(1, 1)),
    );

    let document = app.documentation.selected().document();
    let paragraph = Paragraph::new(document)
        .block(Block::bordered().title("Document"))
        .style(Style::default().fg(Color::Green).bg(Color::Black))
        .scroll((app.documentation_scroll, 0))
        .wrap(Wrap { trim: false });

    render_widget_clamped(frame, paragraph, app.layout.documentation.document);
}

pub fn render_fx(
    effects: &mut Effects,
    layout: &AppLayout,
    prev_tick: Duration,
    frame: &mut Frame,
) {
    render_effect_clamped(
        frame,
        &mut effects.load_document,
        layout.documentation.document.inner(Margin::new(1, 1)),
        prev_tick,
    )
}

#[derive(Debug, Copy, Clone, strum_macros::VariantArray)]
pub enum Document {
    Agents,
    Hud,
    Fabricator,
    Smelter,
    Dog,
    LaserCutter,
    Entities,
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Document::Entities => write!(f, "Entities"),
            Document::Agents => write!(f, "Agents"),
            Document::Hud => write!(f, "  HUD"),
            Document::Fabricator => write!(f, "  Fabricator"),
            Document::Smelter => write!(f, "  Smelter"),
            Document::Dog => write!(f, "  Dog"),
            Document::LaserCutter => write!(f, "  Laser Cutter"),
        }
    }
}

impl Document {
    pub fn document(&self) -> String {
        match self {
            Document::Entities => entities::DOCUMENTATION.to_string(),
            Document::Agents => agents::DOCUMENTATION.to_string(),
            Document::Hud => hud::DOCUMENTATION.to_string(),
            Document::Fabricator => fabricator::DOCUMENTATION.to_string(),
            Document::Smelter => smelter::DOCUMENTATION.to_string(),
            Document::Dog => dog::DOCUMENTATION.to_string(),
            Document::LaserCutter => laser_cutter::DOCUMENTATION.to_string(),
        }
    }
}
