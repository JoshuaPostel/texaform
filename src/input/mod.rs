use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::app::{App, AppResult};
use crate::input::load_game::load_selected_save_file;
use crate::surface::generation;
use crate::ui::Screen;

use std::collections::HashMap;

mod documentation;
mod load_game;
mod main_menu;
mod pause_menu;
mod save_game;
mod settings;
mod surface;
mod tech_tree;

impl Screen {
    pub fn on_load(self, app: &mut App) {
        app.save_button.is_hovered = false;
        app.pause_menu_button.is_hovered = false;
        app.current_research_button.is_hovered = false;
        app.previous_screen_button.button.is_hovered = false;
        match self {
            Screen::SaveGame | Screen::LoadGame => {
                app.fetch_save_files();
                app.save_file_cache = HashMap::new();
                load_selected_save_file(app);
                //                match load_selected_save_file(app) {
                //                    Ok(surface_state) => app.loading_state = LoadingState::Loaded(surface_state),
                //                    Err(e) => {
                //                        tracing::error!("e: {e}");
                //                        app.loading_state = LoadingState::Failed(e.to_string());
                //                    }
                //                };
            }
            Screen::MainMenu => {
                app.surface = generation::empty(app.event_sender.clone());
            }
            _ => (),
        }
    }
}

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(event: KeyEvent, app: &mut App) -> AppResult<()> {
    if (event.code == KeyCode::Char('d') || event.code == KeyCode::Char('D'))
        && event.modifiers == KeyModifiers::CONTROL
    {
        app.quit();
    }
    match app.screen() {
        Screen::MainMenu => main_menu::handle_key_events(event, app).await,
        Screen::PauseMenu => pause_menu::handle_key_events(event, app).await,
        Screen::Settings => settings::handle_key_events(event, app).await,
        Screen::LoadGame => load_game::handle_key_events(event, app).await,
        Screen::SaveGame => save_game::handle_key_events(event, app).await,
        Screen::Surface => surface::handle_key_events(event, app).await,
        Screen::Documentation => documentation::handle_key_events(event, app).await,
        Screen::TechTree => tech_tree::handle_key_events(event, app).await,
    }
}

/// Handles the mouse events and updates the state of [`App`].
pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    use MouseEventKind as Kind;
    let pos = Position {
        x: event.column,
        y: event.row,
    };

    match app.screen() {
        Screen::Settings
        | Screen::PauseMenu
        | Screen::LoadGame
        | Screen::SaveGame
        | Screen::Documentation
        | Screen::TechTree => match event.kind {
            Kind::Moved => {
                app.previous_screen_button.button.is_hovered =
                    app.layout.previous_screen_button.contains(pos);
            }
            Kind::Down(MouseButton::Left) => {
                if app.layout.previous_screen_button.contains(pos) {
                    app.set_screen(*app.previous_screen());
                    // stop handling the event otherwise the previous screen will also handle
                    // the click
                    return Ok(());
                }
            }
            _ => (),
        },
        Screen::MainMenu | Screen::Surface => (),
    }
    match app.screen() {
        Screen::MainMenu => main_menu::handle_mouse_events(event, app).await,
        Screen::PauseMenu => pause_menu::handle_mouse_events(event, app).await,
        Screen::Settings => settings::handle_mouse_events(event, app).await,
        Screen::LoadGame => load_game::handle_mouse_events(event, app).await,
        Screen::SaveGame => save_game::handle_mouse_events(event, app).await,
        Screen::Surface => surface::handle_mouse_events(event, app).await,
        Screen::Documentation => documentation::handle_mouse_events(event, app).await,
        Screen::TechTree => tech_tree::handle_mouse_events(event, app).await,
    }
}

#[derive(Debug)]
pub struct DoubleClickTracker<T: PartialEq> {
    id: Option<T>,
    last_clicked: Instant,
}

impl<T: PartialEq> Default for DoubleClickTracker<T> {
    fn default() -> DoubleClickTracker<T> {
        DoubleClickTracker {
            id: None,
            last_clicked: Instant::now(),
        }
    }
}

impl<T: PartialEq> DoubleClickTracker<T> {
    /// record a click on element `id` and return weather it was a double click
    pub fn clicked(&mut self, id: T) -> bool {
        let was_double_click = if Some(&id) == self.id.as_ref() {
            self.last_clicked.elapsed() < Duration::from_millis(500)
        } else {
            self.id = Some(id);
            false
        };
        self.last_clicked = Instant::now();
        was_double_click
    }
}
