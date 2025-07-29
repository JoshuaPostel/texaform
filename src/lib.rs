/// Application.
pub mod app;

/// Terminal events handler.
pub mod event;

pub mod effects;
/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Event handler.
pub mod input;
pub mod logging;

//pub mod grid;
pub mod surface;
//pub mod generation;

pub mod tcp;
pub mod utils;
pub mod websocket;

pub mod agents;
pub mod entities;

pub mod puzzles;
pub mod tech_tree;

pub mod draw;
pub mod theme;
pub mod widgets;

pub const TICK_UPDATE_MILLS: u64 = 250;
