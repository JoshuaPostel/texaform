use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::Block,
};

use crate::app::App;
use crate::ui::{center, render_widget_clamped};
use crate::widgets::list::TextList;

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

#[derive(Debug, Copy, Clone)]
pub enum PauseMenu {
    Continue,
    SaveGame,
    Documentation,
    TechTree,
    Settings,
    MainMenu,
}

impl PauseMenu {
    pub fn list() -> TextList<PauseMenu> {
        let list = vec![
            PauseMenu::Continue,
            PauseMenu::SaveGame,
            PauseMenu::Documentation,
            PauseMenu::TechTree,
            PauseMenu::Settings,
            PauseMenu::MainMenu,
        ];
        TextList::default_style(list)
    }
}

impl std::fmt::Display for PauseMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PauseMenu::Continue => write!(f, "Continue"),
            PauseMenu::SaveGame => write!(f, "Save Game"),
            PauseMenu::Documentation => write!(f, "Documentation"),
            PauseMenu::TechTree => write!(f, "Technology Tree"),
            PauseMenu::Settings => write!(f, "Settings"),
            PauseMenu::MainMenu => write!(f, "Main Menu"),
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let block = Block::bordered()
        .title("Pause Menu")
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, block, app.layout.whole_screen());

    render_widget_clamped(frame, app.pause_menu.clone(), app.layout.pause_menu.menu);
}
