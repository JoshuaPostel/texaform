use crate::app::{App, AppResult};
use crate::utils::{relative_position, relative_position_bordered};
use crate::widgets::text_box::Action as TextBoxAction;
use crate::widgets::list::Action as ListAction;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::input::load_game::{load_save_file, load_save_file_cached};
use crate::surface::state::SurfaceState;
use crate::widgets::HandleInput;
use crate::widgets::button::{BorderAttachedButton, Location};

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
        },
        Some(ListAction::Choose(path)) => load_save_file(app, &path.inner),
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
    // TODO handle previous_screen button too
    app.save_button.button.is_hovered = app.layout.save_game.save_button.contains(pos);
    if app.save_button.button.is_hovered {
        app.save_files.hover(None);
        return Ok(());
    }
    if let Some(rel_pos) = relative_position(app.layout.save_game.save_file_input, pos) {
        app.save_screen_text_box.handle_mouse_event(event, rel_pos);
    }
    if let Some(rel_pos) = relative_position_bordered(app.layout.save_game.save_files, pos) {
        match app.save_files.handle_mouse_event(event, rel_pos) {
            Some(ListAction::Select(path)) => {
                app.save_screen_text_box.set_content(path.to_string());
                load_save_file_cached(app, &path.inner)
            },
            Some(ListAction::Choose(path)) => load_save_file(app, &path.inner),
            None => (),
        }
    } else {
        app.save_files.hover(None);
    }
    // TODO handle button hover


//    use MouseEventKind as Kind;
//    match event.kind {
//        Kind::Moved => {
//            let pos = Position {
//                x: event.column,
//                y: event.row,
//            };
//            if app.layout.save_game.save_button.contains(pos) {
//                app.save_button.button.is_hovered = true;
//                app.save_files.hover(None);
//            } else if app.layout.save_game.save_files.contains(pos) {
//                let idx = pos
//                    .y
//                    .saturating_sub(1)
//                    .saturating_sub(app.layout.save_game.save_files.y);
//                app.save_files.hover(Some(idx as usize));
//            } else {
//                app.save_files.hover(None);
//            }
//        }
//        Kind::Down(MouseButton::Left) => {
//            if let Some(rel_pos) = relative_position(app.layout.save_game.save_file_input, pos) {
//                app.save_screen_text_box.handle_mouse_event(event, rel_pos);
//            } else {
//                if app.layout.save_game.save_button.contains(pos) {
//                    if let Some(display_pathbuf) = app.save_files.selected()
//                        && let Some(msg) = display_pathbuf.inner.file_name()
//                    {
//                        let save_path =
//                            SurfaceState::save(&app.surface, msg.to_string_lossy().to_string())?;
//                        load_save_file(app, &save_path);
//                        app.save_files.insert(save_path.into());
//                    }
//                } else if app.layout.save_game.save_files.contains(pos) {
//                    let idx = pos
//                        .y
//                        .saturating_sub(1)
//                        .saturating_sub(app.layout.save_game.save_files.y);
//                    app.save_files.select(idx as usize);
//                    if let Some(file) = app.save_files.selected() {
//                        app.save_screen_text_box.set_content(file.to_string());
//                        load_selected_save_file(app);
//                    }
//                }
//            }
//        }
//        _ => (),
//    }
    Ok(())
}
