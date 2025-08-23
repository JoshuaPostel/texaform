use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

use crate::app::{App, AppResult};
use crate::input::load_game::load_selected_save_file;
use crate::surface::generation;
use crate::ui::Screen;

mod documentation;
mod load_game;
mod main_menu;
mod pause_menu;
mod save_game;
mod settings;
mod surface;
mod tech_tree;

// TODO make a trait to keep impl in each screen's mod?
impl Screen {
    pub fn on_load(self, app: &mut App) {
        app.pause_menu_button.hovered(false);
        app.save_button.button.is_hovered = false;
        app.surface.current_research_button.hovered(false);
        app.previous_screen_button.button.is_hovered = false;
        match self {
            Screen::SaveGame | Screen::LoadGame => {
                app.fetch_save_files();
                load_selected_save_file(app);
                if let Some(file) = app.save_files.selected() {
                    app.save_screen_text_box.set_content(file.to_string());
                }
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
