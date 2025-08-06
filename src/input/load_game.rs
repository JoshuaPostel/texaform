use crate::app::{App, AppResult, LoadingState};
use crate::surface;
use crate::surface::state::SurfaceState;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::ui::Screen;

pub fn load_selected_save_file(app: &mut App) {
    if let Some(path) = &app.save_files.selected().map(|x| x.inner.clone()) {
        if !app.save_file_cache.contains_key(path) {
            tracing::info!("attempting to load: {path:?}");
            // TODO find a better way? app.surface should be Option<Surface>?
            //
            // ensure surface is dropped
            // app.surface = Surface::empty(app);
            tracing::info!("pre load");
            // TODO
            let load_result = match SurfaceState::load(path) {
                Ok(surface_state) => LoadingState::Loaded(Box::new(surface_state)),
                Err(e) => {
                    tracing::error!("loading error: {e}");
                    LoadingState::Failed(e.to_string())
                }
            };
            app.save_file_cache.insert(path.clone(), load_result);
            tracing::info!("post load");
        }
    }
}

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.save_files.select_previous();
            load_selected_save_file(app);
            //            match load_selected_save_file(app) { Ok(surface_state) => {
            //                    app.loading_state = LoadingState::Loaded(surface_state);
            //
            //                },
            //                Err(e) => {
            //                    tracing::error!("e: {e}");
            //                    app.loading_state = LoadingState::Failed(e.to_string());
            //                }
            //            };
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.save_files.select_next();
            load_selected_save_file(app);
        }
        KeyCode::Enter => load2(app).await,
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
        Kind::Down(MouseButton::Left) => {
            if app.layout.load_game.save_files.contains(pos) {
                let idx = pos
                    .y
                    .saturating_sub(1)
                    .saturating_sub(app.layout.load_game.save_files.y);
                app.save_files.select(idx as usize);
                load_selected_save_file(app);
                if app.load_game_double_click_tracker.clicked(idx) {
                    load2(app).await
                }
            }
        }
        Kind::Moved => {
            if app.layout.load_game.save_files.contains(pos) {
                let idx = pos
                    .y
                    .saturating_sub(1)
                    .saturating_sub(app.layout.load_game.save_files.y);
                app.save_files.hover(Some(idx as usize));
            } else {
                app.save_files.hover(None);
            }
        }
        _ => (),
    }
    Ok(())
}

async fn load2(app: &mut App) {
    if let Some(path) = &app.save_files.selected().map(|x| x.inner.clone()) {
        if let Some(loading_state) = app.save_file_cache.remove(path) {
            match loading_state {
                LoadingState::Loaded(state) => {
                    tracing::info!("HERE");
                    // TODO how to force agents to be dropped?
                    app.surface = surface::generation::empty(app.event_sender.clone());
                    // TODO the following comment avoids the port in use panic
                    // so its probably a dely issue
                    // need to implement a Comms drop such that it waits till the ports are free
                    // again?
                    tracing::info!("should be dropped?: {:?}", app.surface.agents);
                    app.surface = state.into_surface(app.event_sender.clone()).await;
                    //tracing::info!("surface: {:?}", app.surface);
                    app.set_screen(Screen::Surface);
                }
                LoadingState::Failed(state) => {
                    app.save_file_cache
                        .insert(path.clone(), LoadingState::Failed(state));
                }
                other => {
                    tracing::info!("other: {other:?}")
                }
            }
        }
    }
}

//async fn load(app: &mut App) {
//    let mut loading_state = LoadingState::Loading;
//    std::mem::swap(&mut app.loading_state, &mut loading_state);
//    match loading_state {
//        LoadingState::Loaded(state) => {
//            tracing::info!("HERE");
//            // TODO how to force agents to be dropped?
//            app.surface = Surface::empty(app);
//            // TODO the following comment avoids the port in use panic
//            // so its probably a dely issue
//            // need to implement a Comms drop such that it waits till the ports are free
//            // again?
//            tracing::info!("should be dropped?: {:?}", app.surface.agents);
//            app.surface = state.into_surface(app.event_sender.clone()).await;
//            //tracing::info!("surface: {:?}", app.surface);
//            app.set_screen(Screen::Surface);
//        }
//        other => {
//            tracing::info!("other: {other:?}")
//        }
//    }
//}
