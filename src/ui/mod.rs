pub mod documentation;
mod load_game;
pub mod main_menu;
pub mod pause_menu;
mod save_game;
mod settings;
mod surface;
pub mod tech_tree;

use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{StatefulWidget, Widget},
};
use tachyonfx::EffectRenderer;
use tachyonfx::Shader;

use crate::app::App;

pub fn render_widget_clamped<W: Widget>(frame: &mut Frame, widget: W, area: Rect) {
    frame.render_widget(widget, area.clamp(frame.area()))
}

pub fn render_stateful_widget_clamped<W: StatefulWidget>(
    frame: &mut Frame,
    widget: W,
    area: Rect,
    state: &mut W::State,
) {
    frame.render_stateful_widget(widget, area.clamp(frame.area()), state)
}

pub fn render_effect_clamped<S: Shader>(
    frame: &mut Frame,
    effect: &mut S,
    area: Rect,
    last_tick: std::time::Duration,
) {
    frame.render_effect(effect, area.clamp(frame.area()), last_tick)
}

// source: https://ratatui.rs/recipes/layout/center-a-rect/
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    tracing::info!("area: {area:?}");
    area
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Screen {
    #[default]
    MainMenu,
    PauseMenu,
    Settings,
    LoadGame,
    SaveGame,
    Surface,
    Documentation,
    TechTree,
}

impl std::fmt::Display for Screen {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Screen::MainMenu => write!(f, "Main Menu"),
            Screen::PauseMenu => write!(f, "Pause Menu"),
            Screen::Settings => write!(f, "Settings"),
            Screen::LoadGame => write!(f, "Load Game"),
            Screen::SaveGame => write!(f, "Save Game"),
            Screen::Surface => write!(f, "Surface"),
            Screen::Documentation => write!(f, "Documentation"),
            Screen::TechTree => write!(f, "Technology Tree"),
        }
    }
}

#[derive(Debug, Default)]
pub struct AppLayout {
    pub width: u16,
    pub height: u16,
    pub previous_screen_button: Rect,
    pub main_menu: main_menu::MainMenuLayout,
    pub pause_menu: pause_menu::PauseMenuLayout,
    pub surface: surface::SurfaceLayout,
    pub load_game: load_game::LoadGameLayout,
    pub save_game: save_game::SaveGameLayout,
    pub documentation: documentation::DocumentationLayout,
    pub tech_tree: tech_tree::TechTreeLayout,
}

impl AppLayout {
    pub fn update(width: u16, height: u16, app: &App) -> AppLayout {
        AppLayout {
            width,
            height,
            previous_screen_button: app.previous_screen_button.resize(width, height),
            main_menu: main_menu::MainMenuLayout::new(width, height),
            pause_menu: pause_menu::PauseMenuLayout::new(width, height),
            surface: surface::SurfaceLayout::new(width, height),
            load_game: load_game::LoadGameLayout::new(width, height),
            save_game: save_game::SaveGameLayout::new(width, height, app),
            documentation: documentation::DocumentationLayout::new(width, height, app),
            tech_tree: tech_tree::TechTreeLayout::new(width, height, app),
        }
    }

    pub fn whole_screen(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.screen() {
        Screen::MainMenu => {
            main_menu::render(app, frame);
            main_menu::render_fx(&mut app.effects, &app.layout, app.prev_tick, frame);
        }
        Screen::PauseMenu => pause_menu::render(app, frame),
        Screen::Settings => settings::render(app, frame),
        Screen::LoadGame => load_game::render(app, frame),
        Screen::SaveGame => save_game::render(app, frame),
        Screen::Surface => {
            surface::render(app, frame);
            // TODO
            // app.surface.render_effects();

            //            for effect in app.effects.iter_mut() {
            //                frame.render_effect(
            //                    effect,
            //                    app.layout.whole_screen(),
            //                    app.prev_tick,
            //                )
            //            }
        }
        Screen::Documentation => {
            documentation::render(app, frame);
            documentation::render_fx(&mut app.effects, &app.layout, app.prev_tick, frame);
            // not rendered in documentation::render so that render_fx does not apply to the button
            render_widget_clamped(
                frame,
                &app.previous_screen_button,
                app.layout.previous_screen_button,
            );
            render_widget_clamped(
                frame,
                &app.copy_button,
                app.layout.documentation.copy_button,
            );
        }
        Screen::TechTree => {
            tech_tree::render(app, frame);
        }
    }
}
