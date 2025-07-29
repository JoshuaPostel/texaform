use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::app::App;
use crate::effects::Effects;
use crate::ui::{AppLayout, center, render_effect_clamped, render_widget_clamped};
use crate::widgets::list::TextList;

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

#[derive(Debug, Copy, Clone)]
pub enum MainMenu {
    NewGame,
    LoadGame,
    Settings,
    Exit,
}

impl MainMenu {
    pub fn list() -> TextList<MainMenu> {
        let list = vec![
            MainMenu::NewGame,
            MainMenu::LoadGame,
            MainMenu::Settings,
            MainMenu::Exit,
        ];
        TextList::default_style(list)
    }
}

impl std::fmt::Display for MainMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MainMenu::NewGame => write!(f, "New Game"),
            MainMenu::LoadGame => write!(f, "Load Game"),
            MainMenu::Settings => write!(f, "Settings"),
            MainMenu::Exit => write!(f, "Exit [CTRL + D]"),
        }
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
