use crate::{app::{App, AppResult}, widgets::HandleInput};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Margin, Position};
use tachyonfx::Shader;
use crate::widgets::list::Action as ListAction;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match app.documentation.handle_key_event(key_event) {
        Some(ListAction::Select(document)) | Some(ListAction::Choose(document)) => {
            app.documentation.select_by_display(&document);
            app.documentation_scroll = 0;
            app.effects.load_document.reset()
        },
        _ => (),
    }
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                let mut clipboard = Clipboard::new().expect("can access clipboard");
                let document = app.documentation.selected_unchecked().document();
                clipboard.set_text(document).expect("can set clipboard");
            }
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

    // TODO add to layout
    let doc_list = app.layout.documentation.list.inner(Margin::new(1, 1));
    match app.documentation.handle_mouse(doc_list, pos, event) {
        Some(ListAction::Select(document)) | Some(ListAction::Choose(document)) => {
            app.documentation.select_by_display(&document);
            app.documentation_scroll = 0;
            app.effects.load_document.reset()
        },
        _ => (),
    }
    use MouseEventKind as Kind;
    match event.kind {
        Kind::ScrollDown => {
            app.documentation_scroll = app.documentation_scroll.saturating_add(1);
        }
        Kind::ScrollUp => {
            app.documentation_scroll = app.documentation_scroll.saturating_sub(1);
        }
        Kind::Moved => {
            app.copy_button.button.is_hovered = app.layout.documentation.copy_button.contains(pos);
        }
        Kind::Down(MouseButton::Left) => {
            if app.layout.documentation.copy_button.contains(pos) {
                let mut clipboard = Clipboard::new().expect("can access clipboard");
                let document = app.documentation.selected_unchecked().document();
                clipboard.set_text(document).expect("can set clipboard");
            }
        }
        _ => (),
    }
    Ok(())
}
