use crate::app::{App, AppResult};
use crate::utils::relative_position;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::input::load_game::{load_save_file, load_selected_save_file};
use crate::surface::state::SurfaceState;
use crate::widgets::button::{BorderAttachedButton, Location};
use crate::widgets::HandleInput;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if let Some(msg) = app.save_screen_text_box.handle_key_event(key_event) {
        let save_path = SurfaceState::save(&app.surface, msg.clone())?;
        load_save_file(app, &save_path);
        app.save_files.insert(save_path.into());
    }
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        KeyCode::Up => {
            app.save_files.select_previous();
            if let Some(file) = app.save_files.selected() {
                app.save_screen_text_box.set_content(file.to_string());
            }
            load_selected_save_file(app);
        }
        KeyCode::Down => {
            app.save_files.select_next();
            if let Some(file) = app.save_files.selected() {
                app.save_screen_text_box.set_content(file.to_string());
            }
            load_selected_save_file(app);
        }
        _ => {}
    }
    let label = if app
        .save_files
        .items
        .iter()
        .any(|i| i.to_string() == app.save_screen_text_box.content())
    {
        " Overwrite [ENTER] "
    } else {
        "    Save [ENTER]   "
    };
    app.save_button = BorderAttachedButton::new(label.to_string(), Location::East(6));
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
            let pos = Position {
                x: event.column,
                y: event.row,
            };
            if app.layout.save_game.save_button.contains(pos) {
                app.save_button.button.is_hovered = true;
                app.save_files.hover(None);
            } else if app.layout.save_game.save_files.contains(pos) {
                let idx = pos
                    .y
                    .saturating_sub(1)
                    .saturating_sub(app.layout.save_game.save_files.y);
                app.save_files.hover(Some(idx as usize));
            } else {
                app.save_files.hover(None);
            }
        }
        Kind::Down(MouseButton::Left) => {
            if let Some(rel_pos) = relative_position(app.layout.save_game.save_file_input, pos) {
                app.save_screen_text_box.handle_mouse_event(event, rel_pos);
                // TODO
                
                //app.layout.save_game.save_file_input.

                //app.input_mode = InputMode::Editing;
            } else {
                //app.input_mode = InputMode::Normal;

                if app.layout.save_game.save_button.contains(pos) {
                    if let Some(display_pathbuf) = app.save_files.selected()
                        && let Some(msg) = display_pathbuf.inner.file_name()
                    {
                        let save_path =
                            SurfaceState::save(&app.surface, msg.to_string_lossy().to_string())?;
                        load_save_file(app, &save_path);
                        app.save_files.insert(save_path.into());
                        //app.fetch_save_files();
                    }
                } else if app.layout.save_game.save_files.contains(pos) {
                    let idx = pos
                        .y
                        .saturating_sub(1)
                        .saturating_sub(app.layout.save_game.save_files.y);
                    app.save_files.select(idx as usize);
                    if let Some(file) = app.save_files.selected() {
                        app.save_screen_text_box.set_content(file.to_string());
                        load_selected_save_file(app);
                    }
                }
            }
        }
        _ => (),
    }
    Ok(())
}
