use ratatui::prelude::Alignment;
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, block::Title},
};
use strum::IntoEnumIterator;

use crate::effects::Effects;
use crate::surface::Seed;
use crate::ui::{AppLayout, center, render_effect_clamped, render_widget_clamped};
use crate::widgets::list::TextList;
use crate::{app::App, widgets::list::AlignedLine};

use std::time::Duration;

#[derive(Debug, Default)]
pub struct MainMenuLayout {
    pub menu: Rect,
    pub set_seed: Rect,
}

impl MainMenuLayout {
    pub fn new(width: u16, height: u16) -> MainMenuLayout {
        let area = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        let menu = center(area, Constraint::Length(25), Constraint::Length(4));

        let set_seed = Rect {
            x: menu.x,
            y: menu.bottom() + 2,
            width: menu.width,
            height: 3,
        };
        MainMenuLayout { menu, set_seed }
    }
}

#[derive(Debug, Copy, Clone, strum_macros::Display, strum_macros::EnumIter)]
#[strum(serialize_all = "title_case")]
pub enum MainMenu {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

impl MainMenu {
    pub fn list() -> TextList<MainMenu> {
        let list = MainMenu::iter().collect();
        let lines: Vec<AlignedLine> = vec![
            AlignedLine::from(MainMenu::NewGame.to_string()),
            AlignedLine::from(MainMenu::LoadGame.to_string()),
            AlignedLine::from(MainMenu::Settings.to_string()),
            AlignedLine::left_right(MainMenu::Exit.to_string(), "[CTRL + D]".to_string()),
        ];
        TextList::default_style_with_lines(list, lines)
    }
}

fn render_set_seed(app: &App, frame: &mut Frame) {
    let content = match app.seed {
        Seed::Manual(x) => format!("{x:0>6}"),
        Seed::Random(_) => "Random".to_string(),
    };
    let seed_input = Paragraph::new(content).alignment(Alignment::Center).block(
        Block::default()
            .title("Set Seed")
            .title(Title::from("[0-9, DEL]").alignment(Alignment::Right))
            .borders(Borders::ALL),
    );

    render_widget_clamped(frame, seed_input, app.layout.main_menu.set_seed);
}

pub fn render(app: &App, frame: &mut Frame) {
    let logo = indoc::indoc! {"\n\n\n
        ░        ░░        ░░  ░░░░  ░░░      ░░░        ░░░      ░░░       ░░░  ░░░░  ░
        ▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒▒▒  ▒▒  ▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒▒   ▒▒   ▒
        ▓▓▓▓  ▓▓▓▓▓      ▓▓▓▓▓▓    ▓▓▓▓  ▓▓▓▓  ▓▓      ▓▓▓▓  ▓▓▓▓  ▓▓       ▓▓▓        ▓
        ████  █████  █████████  ██  ███        ██  ████████  ████  ██  ███  ███  █  █  █
        ████  █████        ██  ████  ██  ████  ██  █████████      ███  ████  ██  ████  █
    "};
    let logo = Paragraph::new(logo)
        .centered()
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, logo, app.layout.whole_screen());
    render_widget_clamped(frame, app.main_menu.clone(), app.layout.main_menu.menu);
    render_set_seed(app, frame);
}

pub fn render_fx(
    effects: &mut Effects,
    layout: &AppLayout,
    prev_tick: Duration,
    frame: &mut Frame,
) {
    render_effect_clamped(
        frame,
        &mut effects.main_menu_logo,
        layout.whole_screen(),
        prev_tick,
    );
}
