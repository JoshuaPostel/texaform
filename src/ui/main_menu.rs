use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use strum::IntoEnumIterator;

use crate::effects::Effects;
use crate::ui::{AppLayout, center, render_effect_clamped, render_widget_clamped};
use crate::widgets::list::TextList;
use crate::{app::App, widgets::list::AlignedLine};

use std::time::Duration;

#[derive(Debug, Default)]
pub struct MainMenuLayout {
    pub menu: Rect,
}

impl MainMenuLayout {
    pub fn new(width: u16, height: u16) -> MainMenuLayout {
        let area = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        MainMenuLayout {
            menu: center(area, Constraint::Percentage(20), Constraint::Length(4)),
        }
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

pub fn render(app: &App, frame: &mut Frame) {
    let logo = indoc::indoc! {"\n\n\n
        ░        ░░        ░░  ░░░░  ░░░      ░░░        ░░░      ░░░       ░░░  ░░░░  ░
        ▒▒▒▒  ▒▒▒▒▒  ▒▒▒▒▒▒▒▒▒  ▒▒  ▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒▒▒▒▒  ▒▒▒▒  ▒▒  ▒▒▒▒  ▒▒   ▒▒   ▒
        ▓▓▓▓  ▓▓▓▓▓      ▓▓▓▓▓▓    ▓▓▓▓  ▓▓▓▓  ▓▓      ▓▓▓▓  ▓▓▓▓  ▓▓       ▓▓▓        ▓
        ████  █████  █████████  ██  ███        ██  ████████  ████  ██  ███  ███  █  █  █
        ████  █████        ██  ████  ██  ████  ██  █████████      ███  ████  ██  ████  █
    "};
    let paragraph = Paragraph::new(logo)
        .centered()
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, paragraph, app.layout.whole_screen());
    render_widget_clamped(frame, app.main_menu.clone(), app.layout.main_menu.menu);
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
