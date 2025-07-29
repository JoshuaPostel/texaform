use crate::app::{App, AppResult};
use crate::ui::Screen;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::ui::pause_menu::PauseMenu;

fn on_select(app: &mut App) {
    match app.pause_menu.selected() {
        PauseMenu::Continue => app.set_screen(Screen::Surface),
        PauseMenu::SaveGame => app.set_screen(Screen::SaveGame),
        PauseMenu::Documentation => app.set_screen(Screen::Documentation),
        PauseMenu::TechTree => app.set_screen(Screen::TechTree),
        PauseMenu::Settings => app.set_screen(Screen::Settings),
        PauseMenu::MainMenu => app.set_screen(Screen::MainMenu),
    }
}

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => app.set_screen(Screen::Surface),
        KeyCode::Up | KeyCode::Char('k') => app.pause_menu.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.pause_menu.select_next(),
        KeyCode::Enter => on_select(app),
        _ => {}
    }
    Ok(())
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    let pos = Position {
        x: event.column,
        y: event.row,
    };
    use MouseEventKind as Kind;
    match event.kind {
        Kind::Moved => {
            if app.layout.pause_menu.menu.contains(pos) {
                let idx = pos.y.saturating_sub(app.layout.pause_menu.menu.y);
                app.pause_menu.select(idx as usize);
            }
        }
        // TODO BUG: does not line up with cursor
        Kind::Down(MouseButton::Left) => {
            if app.layout.pause_menu.menu.contains(pos) {
                on_select(app)
            }
        }
        _ => (),
    }
    Ok(())
}
