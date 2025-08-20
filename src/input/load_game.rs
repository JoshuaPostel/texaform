use std::path::PathBuf;

use crate::app::{App, AppResult, LoadingState};
use crate::surface;
use crate::surface::state::SurfaceState;
use crate::utils::relative_position_bordered;
use crate::widgets::HandleInput;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use crate::widgets::list::Action;

use crate::ui::Screen;

pub fn load_selected_save_file(app: &mut App) {
    if let Some(path) = &app.save_files.selected().map(|x| x.inner.clone()) {
        load_save_file_cached(app, path)
    }
}

pub fn load_save_file_cached(app: &mut App, path: &PathBuf) {
    if !app.save_file_cache.contains_key(path) {
        load_save_file(app, path)
    }
}

pub fn load_save_file(app: &mut App, path: &PathBuf) {
    tracing::info!("attempting to load: {path:?}");
    // TODO https://www.youtube.com/watch?list=RDOi0sVRZ_49c&v=JynLgzm-Emwnd a better way? app.surface should be Option<Surface>?
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

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.save_files.handle_key_event(key_event) {
        Some(Action::Select(path)) => load_save_file_cached(app, &path.inner),
        Some(Action::Choose(path)) => load(app, &path.inner).await,
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
    if let Some(rel_pos) = relative_position_bordered(app.layout.load_game.save_files, pos) {
        match app.save_files.handle_mouse_event(event, rel_pos) {
            Some(Action::Select(path)) => load_save_file_cached(app, &path.inner),
            Some(Action::Choose(path)) => load(app, &path.inner).await,
            None => (),
        }
    } else {
        app.save_files.hover(None);
    }
    Ok(())
}

async fn load(app: &mut App, path: &PathBuf) {
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
