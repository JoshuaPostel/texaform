use crate::app::{App, AppResult};
use crate::input::Screen;
use crate::surface::{self, AddEntityError};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::ui::main_menu::MainMenu;

//async fn on_select(app: &mut App) -> AppResult<()> {
async fn on_select(app: &mut App) -> Result<(), AddEntityError> {
    match app.main_menu.selected() {
        MainMenu::NewGame => {
            //app.surface = surface::generation::sparse_xs(app.event_sender.clone());
            //app.surface = surface::generation::perlin(app.event_sender.clone());
            app.surface =
                surface::generation::manual(app.event_sender.clone(), app.seed.value()).await;
            //surface::generation::init_some_agents(&mut app.surface).await?;
            //surface::generation::init_starting_agent(&mut app.surface).await?;
            surface::generation::init_starting_entities(&mut app.surface).await?;
            app.set_screen(Screen::Surface)
        }
        MainMenu::LoadGame => app.set_screen(Screen::LoadGame),
        MainMenu::Settings => app.set_screen(Screen::Settings),
        MainMenu::Exit => app.quit(),
    }
    Ok(())
}

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => app.main_menu.select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.main_menu.select_next(),
        KeyCode::Enter => on_select(app).await?,
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
            if app.layout.main_menu.menu.contains(pos) {
                let idx = pos.y.saturating_sub(app.layout.main_menu.menu.y);
                app.main_menu.select(idx as usize);
            }
        }
        Kind::Down(MouseButton::Left) => {
            // TODO how to handle mouse outside of menu area?
            if app.layout.main_menu.menu.contains(pos) {
                on_select(app).await?;
            }
        }
        _ => (),
    }
    Ok(())
}
