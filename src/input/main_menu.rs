use crate::app::{App, AppResult};
use crate::input::Screen;
use crate::surface::state::Seed;
use crate::surface::{self, AddEntityError};
use crate::widgets::HandleInput;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Position;

use crate::ui::main_menu::MainMenu;

//async fn on_select(app: &mut App) -> AppResult<()> {
async fn on_select(app: &mut App, screen: MainMenu) -> Result<(), AddEntityError> {
    match screen {
        MainMenu::NewGame => {
            // make sure random seed is new
            if matches!(app.seed, Seed::Random(_)) {
                app.seed = Seed::default();
            }
            app.surface = surface::generation::manual(app.event_sender.clone(), app.seed).await;
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
    if let Some(screen) = app.main_menu.handle_key_event(key_event) {
        on_select(app, screen).await?;
    }
    match key_event.code {
        KeyCode::Char(x) => {
            if let Some(digit) = x.to_digit(10) {
                app.seed.append(digit as u64)
            }
        }
        KeyCode::Delete | KeyCode::Backspace => app.seed.backspace(),
        _ => {}
    }
    Ok(())
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    let pos = Position {
        x: event.column,
        y: event.row,
    };
    if let Some(screen) = app
        .main_menu
        .handle_mouse(app.layout.main_menu.menu, pos, event)
    {
        on_select(app, screen).await?;
    }
    Ok(())
}
