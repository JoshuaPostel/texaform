use crate::app::{App, AppResult};
use arboard::Clipboard;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Position;
use tachyonfx::Shader;

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => {
            app.set_screen(*app.previous_screen());
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.documentation.select_previous();
            app.documentation_scroll = 0;
            app.effects.load_document.reset()
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.documentation.select_next();
            app.documentation_scroll = 0;
            app.effects.load_document.reset()
            //app.effects[1].done()
            //app.effects[1].set_area()
            //app.effects[1].cloned_box() -> Box<dyn Shader>
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
    use MouseEventKind as Kind;
    match event.kind {
        Kind::ScrollDown => {
            app.documentation_scroll = app.documentation_scroll.saturating_add(1);
        }
        Kind::ScrollUp => {
            app.documentation_scroll = app.documentation_scroll.saturating_sub(1);
        }
        Kind::Moved => {
            if app.layout.documentation.list.contains(pos) {
                let idx = pos
                    .y
                    .saturating_sub(1)
                    .saturating_sub(app.layout.documentation.list.y);
                app.documentation.hover(Some(idx as usize));
            } else {
                app.documentation.hover(None);
            }
            app.copy_button.button.is_hovered = app.layout.documentation.copy_button.contains(pos);
        }
        Kind::Down(MouseButton::Left) => {
            if app.layout.documentation.list.contains(pos) {
                let idx = pos
                    .y
                    .saturating_sub(1)
                    .saturating_sub(app.layout.documentation.list.y);
                app.documentation.select(idx as usize);
                app.documentation_scroll = 0;
            }
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
