use crate::app::{App, AppResult, InputMode};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use tracing::*;

use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;

use crate::surface::grid::Gent;
use crate::surface::{Focus, tutorial::Tutorial};
use crate::ui::Screen;
use crate::widgets::HandleInput;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    let focused_comms = app.surface.focused_agent_comms_mut();
    match app.input_mode {
        InputMode::Editing => {
            if key_event.code == KeyCode::Esc {
                app.input_mode = InputMode::Normal;
            }
            if let Some(comms) = focused_comms
                && let Some(msg) = comms.text_box.handle_key_event(key_event)
            {
                let port = comms.port;
                app.surface.update_agent_manual(&port, msg).await;
            }
        }
        InputMode::Normal => {
            match key_event.code {
                KeyCode::Right => app.surface.move_right(1),
                KeyCode::Left => app.surface.move_left(1),
                KeyCode::Up => app.surface.move_up(1),
                KeyCode::Down => app.surface.move_down(1),
                KeyCode::End => app.surface.move_right(10),
                KeyCode::Home => app.surface.move_left(10),
                KeyCode::PageUp => app.surface.move_up(10),
                KeyCode::PageDown => app.surface.move_down(10),
                KeyCode::Char('D') | KeyCode::Char('d') => {
                    app.set_screen(Screen::Documentation);
                }
                KeyCode::Char('M') | KeyCode::Char('m') => {
                    app.set_screen(Screen::PauseMenu);
                }
                KeyCode::Char('T') | KeyCode::Char('t') => {
                    app.set_screen(Screen::TechTree);
                }
                KeyCode::Char('C') | KeyCode::Char('c') => {
                    if let Some(comms) = focused_comms
                        && comms.address.is_none()
                    {
                        app.input_mode = InputMode::Editing;
                    }
                }
                KeyCode::Char('N') | KeyCode::Char('n') => {
                    app.surface.game_state.tutorial_state.next();
                }
                KeyCode::Char('P') | KeyCode::Char('p') => {
                    app.surface.game_state.tutorial_state.previous();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    use MouseEventKind as Kind;
    let pos = Position {
        x: event.column,
        y: event.row,
    };
    match event.kind {
        Kind::Moved => {
            app.pause_menu_button.is_hovered = app.layout.surface.pause_menu_button.contains(pos);
            app.current_research_button.is_hovered = app.layout.surface.tech.contains(pos);
            if app.surface.game_state.tutorial_state != Tutorial::Complete {
                app.tutorial_previous_button.is_hovered =
                    app.layout.surface.tutorial.previous_button.contains(pos);
                app.tutorial_next_button.is_hovered =
                    app.layout.surface.tutorial.next_button.contains(pos);
            }
        }
        Kind::Down(MouseButton::Left) => {
            info!("clicked: col {}, row {}", event.column, event.row);
            match app.layout.surface.agent.text_box {
                Some(area) if area.contains(pos) => {
                    app.input_mode = InputMode::Editing;
                    return Ok(());
                }
                _ => {
                    app.input_mode = InputMode::Normal;
                }
            }
            if app.layout.surface.surface.contains(pos) {
                let grid_pos = app.surface.grid_position(&pos);
                match app.surface.grid.get_direct(&grid_pos) {
                    Some(Gent::Intmd(_)) => {
                        app.surface.focus = Some(Focus::Position(grid_pos));
                    }
                    Some(Gent::Large(root_pos)) => {
                        if let Some(Gent::Age(_)) = app.surface.grid.get(&grid_pos) {
                            if let Some(port) = app.surface.agent_port(&grid_pos) {
                                app.surface.focus = Some(Focus::Agent(port));
                            } else {
                                tracing::warn!("expected agent at {grid_pos:?}");
                            }
                        } else {
                            app.surface.focus = Some(Focus::Position(*root_pos));
                        }
                    }
                    Some(Gent::Age(_)) => {
                        if let Some(port) = app.surface.agent_port(&grid_pos) {
                            app.surface.focus = Some(Focus::Agent(port));
                        } else {
                            tracing::warn!("expected agent at {grid_pos:?}");
                        }
                    }
                    Some(Gent::Empty) => app.surface.focus = Some(Focus::Position(grid_pos)),
                    Some(_) => tracing::info!("Some(_) branch"),
                    None => tracing::info!("None branch"),
                }
            }
            if app.layout.surface.tech.contains(pos) {
                app.set_screen(Screen::TechTree);
            }
            if app.layout.surface.agents.contains(pos) {
                let index = (pos.y - app.layout.surface.agents.y)
                    .checked_sub(1)
                    .map(usize::from);
                if let Some(idx) = index {
                    if let Some(port) = app.surface.agents.keys().nth(idx) {
                        app.surface.focus = Some(Focus::Agent(*port));
                    }
                }
            }
            if app.layout.surface.pause_menu_button.contains(pos) {
                if let Ok(file) = File::open("assets/beep2.wav") {
                    let reader = BufReader::new(file);
                    let source = Decoder::new(reader).unwrap();

                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source);
                    sink.play();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    sink.detach();
                }
                app.set_screen(Screen::PauseMenu);

                return Ok(());
            }
            if app.surface.game_state.tutorial_state != Tutorial::Complete {
                if app.layout.surface.tutorial.previous_button.contains(pos) {
                    app.surface.game_state.tutorial_state.previous();
                } else if app.layout.surface.tutorial.next_button.contains(pos) {
                    app.surface.game_state.tutorial_state.next();
                }
            }
        }
        _ => (),
    }
    Ok(())
}
