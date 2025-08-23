use crate::app::{App, AppResult};
use crate::widgets::list::Action as ListAction;
use crate::widgets::text_box::Action as TextBoxAction;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Margin, Position};

use crate::input::load_game::{load_save_file, load_save_file_cached};
use crate::surface::state::SurfaceState;
use crate::widgets::HandleInput;
use crate::widgets::depreciated_button::{BorderAttachedButton, Location};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.save_screen_text_box.handle_key_event(key_event) {
        Some(TextBoxAction::Submit(msg)) => {
            let save_path = SurfaceState::save(&app.surface, msg.clone())?;
            load_save_file(app, &save_path);
            app.save_files.insert(save_path.into());
        }
        Some(TextBoxAction::Edit(content)) => {
            let label = if app.save_files.select_by_display(&content) {
                " Overwrite [ENTER] "
            } else {
                "    Save [ENTER]   "
            };
            app.save_button = BorderAttachedButton::new(label.to_string(), Location::East(6));
        }
        None => (),
    }
    match app.save_files.handle_key_event(key_event) {
        Some(ListAction::Select(path)) => {
            app.save_screen_text_box.set_content(path.to_string());
            load_save_file_cached(app, &path.inner)
        }
        Some(ListAction::Choose(path)) => {
            SurfaceState::save_to_path(&app.surface, &path.inner)?;
            load_save_file(app, &path.inner)
        },
        None => (),
    }
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    let pos = Position {
        x: event.column,
        y: event.row,
    };
    app.save_button.button.is_hovered = app.layout.save_game.save_button.contains(pos);
    if app.save_button.button.is_hovered {
        if event.kind == MouseEventKind::Down(MouseButton::Left)
            && let Some(path) = app.save_files.selected()
        {
            SurfaceState::save_to_path(&app.surface, &path.inner)?;
            load_save_file(app, &path.clone().inner);
            app.save_screen_text_box.set_content("".to_string());
        }
        app.save_files.hover(None);
        return Ok(());
    }
    if app.layout.previous_screen_button.contains(pos) {
        app.save_files.hover(None);
        return Ok(());
    }
    app.save_screen_text_box
        .handle_mouse(app.layout.save_game.save_file_input, pos, event);

    // TODO store this in layout
    let save_files_list = app.layout.save_game.save_files.inner(Margin::new(1, 1));
    match app.save_files.handle_mouse(save_files_list, pos, event) {
        Some(ListAction::Select(path)) => {
            app.save_screen_text_box.set_content(path.to_string());
            load_save_file_cached(app, &path.inner)
        }
        Some(ListAction::Choose(path)) => {
            SurfaceState::save_to_path(&app.surface, &path.inner)?;
            load_save_file(app, &path.inner);
            app.save_screen_text_box.set_content("".to_string());
        },
        None => (),
    }
    Ok(())
}
