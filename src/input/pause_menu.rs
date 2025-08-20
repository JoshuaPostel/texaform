use crate::app::{App, AppResult};
use crate::ui::Screen;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::utils::relative_position;

use crate::ui::pause_menu::PauseMenu;
use crate::widgets::HandleInput;

fn on_select(app: &mut App, screen: PauseMenu) {
    match screen {
        PauseMenu::Continue => app.set_screen(Screen::Surface),
        PauseMenu::SaveGame => app.set_screen(Screen::SaveGame),
        PauseMenu::Documentation => app.set_screen(Screen::Documentation),
        PauseMenu::TechnologyTree => app.set_screen(Screen::TechTree),
        PauseMenu::Settings => app.set_screen(Screen::Settings),
        PauseMenu::MainMenu => app.set_screen(Screen::MainMenu),
    }
}

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if let Some(screen) = app.pause_menu.handle_key_event(key_event) {
        on_select(app, screen);
    }
    match key_event.code {
        KeyCode::Esc => app.set_screen(Screen::Surface),
        KeyCode::Char('D') | KeyCode::Char('d') => {
            app.set_screen(Screen::Documentation);
        }
        KeyCode::Char('T') | KeyCode::Char('t') => {
            app.set_screen(Screen::TechTree);
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_mouse_events(mouse_event: MouseEvent, app: &mut App) -> AppResult<()> {
    let pos = Position {
        x: mouse_event.column,
        y: mouse_event.row,
    };
    if let Some(rel_pos) = relative_position(app.layout.pause_menu.menu, pos)
        && let Some(screen) = app.pause_menu.handle_mouse_event(mouse_event, rel_pos)
    {
        on_select(app, screen);
    }
    Ok(())
}
