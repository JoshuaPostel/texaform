use crate::app::{App, AppResult, InputMode};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::input::load_game::load_selected_save_file;
use crate::widgets::text_box::TextBox;

use crate::ui::Screen;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.input_mode {
        InputMode::Editing => {
            let text_box = &mut app.save_screen_text_box;
            match key_event.code {
                KeyCode::Esc => {
                    // TODO why is there delay/lag in this branch?
                    app.input_mode = InputMode::Normal;
                }
                KeyCode::Enter => {
                    // TODO
                    //app.surface.update_agent_manual(&port).await;
                    tracing::info!("surface pre save: {:?}", app.surface);
                    app.surface.save(app.save_screen_text_box.input.clone());
                    app.set_screen(Screen::PauseMenu);
                }
                KeyCode::Char(to_insert) => {
                    text_box.enter_char(to_insert);
                }
                KeyCode::Backspace => {
                    text_box.delete_char();
                }
                KeyCode::Left => {
                    text_box.move_cursor_left();
                }
                KeyCode::Right => {
                    text_box.move_cursor_right();
                }
                _ => {}
            }
        }
        InputMode::Normal => match key_event.code {
            KeyCode::Esc => {
                app.set_screen(*app.previous_screen());
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.save_files.select_previous();
                if let Some(file) = app.save_files.selected() {
                    app.save_screen_text_box.input = file.to_string();
                }
                load_selected_save_file(app);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.save_files.select_next();
                if let Some(file) = app.save_files.selected() {
                    app.save_screen_text_box.input = file.to_string();
                }
                load_selected_save_file(app);
            }
            KeyCode::Enter => {
                tracing::info!("surface pre save: {:?}", app.surface);
                app.surface.save(app.save_screen_text_box.input.clone());
                app.set_screen(Screen::PauseMenu);
            }
            _ => {}
        },
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
            let pos = Position {
                x: event.column,
                y: event.row,
            };
            app.save_button.is_hovered = app.layout.save_game.save_button.contains(pos);
            if app.layout.save_game.save_files.contains(pos) {
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
            if app.layout.save_game.save_file_input.contains(pos) {
                app.input_mode = InputMode::Editing;
            } else {
                app.input_mode = InputMode::Normal;

                if app.layout.save_game.save_files.contains(pos) {
                    let idx = pos
                        .y
                        .saturating_sub(1)
                        .saturating_sub(app.layout.save_game.save_files.y);
                    app.save_files.select(idx as usize);
                    if let Some(file) = app.save_files.selected() {
                        app.save_screen_text_box = TextBox::new(file.to_string());
                        load_selected_save_file(app);
                    }
                }
            }
        }
        _ => (),
    }
    Ok(())
}
