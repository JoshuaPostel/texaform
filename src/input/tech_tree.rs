use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        KeyCode::Enter => {
            let tech_tree = &mut app.surface.game_state.tech_tree;
            // TODO report error in UI
            let _todo = tech_tree.set_research(tech_tree.selected_node);
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
    use MouseEventKind as Kind;
    if let Kind::Down(MouseButton::Left) = event.kind
        && app.layout.tech_tree.tree.contains(pos)
        && let Some(idx) = app
            .layout
            .tech_tree
            .nodes
            .iter()
            .position(|node_area| node_area.contains(pos))
    {
        // TODO move selected_node elsewhere when we refactor/optimze rendering
        //let tech_tree = &mut app.surface.game_state.write().expect("TODO").tech_tree;
        let tech_tree = &mut app.surface.game_state.tech_tree;
        tech_tree.selected_node = idx;
        if app.tech_tree_double_click_tracker.clicked(idx) {
            // TODO report error in UI
            let _todo = tech_tree.set_research(tech_tree.selected_node);
        }
    }
    Ok(())
}
