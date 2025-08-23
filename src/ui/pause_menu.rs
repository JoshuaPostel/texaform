use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Block,
};
use strum::IntoEnumIterator;

use crate::app::App;
use crate::ui::{center, render_widget_clamped};
use crate::widgets::list::{AlignedLine, ClickList};

#[derive(Debug, Default)]
pub struct PauseMenuLayout {
    pub menu: Rect,
}

impl PauseMenuLayout {
    pub fn new(width: u16, height: u16) -> PauseMenuLayout {
        let area = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        PauseMenuLayout {
            menu: center(area, Constraint::Percentage(20), Constraint::Length(6)),
        }
    }
}

#[derive(Debug, Copy, Clone, strum_macros::Display, strum_macros::EnumIter)]
#[strum(serialize_all = "title_case")]
pub enum PauseMenu {
    Continue,
    SaveGame,
    Documentation,
    TechnologyTree,
    Settings,
    MainMenu,
}

impl PauseMenu {
    pub fn list() -> ClickList<PauseMenu> {
        let list = PauseMenu::iter().collect();
        let lines: Vec<AlignedLine> = vec![
            AlignedLine::left_right(PauseMenu::Continue.to_string(), "[ESC]".to_string()),
            AlignedLine::from(PauseMenu::SaveGame.to_string()),
            AlignedLine::left_right(PauseMenu::Documentation.to_string(), "[D]".to_string()),
            AlignedLine::left_right(PauseMenu::TechnologyTree.to_string(), "[T]".to_string()),
            AlignedLine::from(PauseMenu::Settings.to_string()),
            AlignedLine::from(PauseMenu::MainMenu.to_string()),
        ];
        let mut list = ClickList::default_style_with_lines(list, lines);
        list.select(0);
        list
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let block = Block::bordered()
        .title("Menu")
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, block, app.layout.whole_screen());

    render_widget_clamped(frame, &app.pause_menu, app.layout.pause_menu.menu);
}
