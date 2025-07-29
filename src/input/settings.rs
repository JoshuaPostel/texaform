use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if key_event.code == KeyCode::Esc {
        app.set_screen(*app.previous_screen());
    }
    Ok(())
}

pub async fn handle_mouse_events(_event: MouseEvent, _app: &mut App) -> AppResult<()> {
    Ok(())
}
