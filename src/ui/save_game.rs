use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Position, Rect},
    style::{Color, Stylize},
    widgets::Block,
};

use crate::app::App;
use crate::ui::load_game::{
    render_save_file_metadata, render_save_file_preview, save_file_boarder,
};
use crate::ui::render_widget_clamped;
use crate::utils::relative_position;

#[derive(Debug, Default)]
pub struct SaveGameLayout {
    pub save_files: Rect,
    pub save_file_input: Rect,
    pub surface_preview: Rect,
    pub metadata: Rect,
    pub save_button: Rect,
}

#[allow(dead_code)]
pub enum SaveGameArea {
    SaveFiles(Position),
    SaveFileInput(Position),
    SurfacePreview(Position),
    Metadata(Position),
    SaveButton(Position),
}

impl SaveGameLayout {
    // this might make handle_mouse_event nicer to implement
    #[allow(dead_code)]
    fn sketch_of_macro(self, position: Position) -> Option<SaveGameArea> {
        if let Some(rel_pos) = relative_position(self.save_files, position) {
            return Some(SaveGameArea::SaveFiles(rel_pos));
        }
        if let Some(rel_pos) = relative_position(self.save_file_input, position) {
            return Some(SaveGameArea::SaveFileInput(rel_pos));
        }
        None
        // todo cover all
    }

    pub fn new(width: u16, height: u16, app: &App) -> SaveGameLayout {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(Rect {
                x: 0,
                y: 0,
                width,
                height,
            });

        let left_column =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(columns[0]);

        let right_column =
            Layout::vertical([Constraint::Max(3), Constraint::Fill(1)]).split(columns[1]);

        let save_button = app.save_button.resize(width, height);

        SaveGameLayout {
            save_file_input: right_column[0],
            save_files: right_column[1],
            surface_preview: left_column[0],
            metadata: left_column[1],
            save_button,
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let border = save_file_boarder();
    render_widget_clamped(frame, border, app.layout.save_game.save_files);

    render_widget_clamped(
        frame,
        &app.save_files,
        app.layout.save_game.save_files.inner(Margin::new(1, 1)),
    );

    let block = Block::bordered()
        .title("Name")
        .fg(Color::Green)
        .bg(Color::Black);
    render_widget_clamped(frame, block, app.layout.save_game.save_file_input);
    render_widget_clamped(
        frame,
        &app.save_screen_text_box,
        app.layout
            .save_game
            .save_file_input
            .inner(Margin::new(1, 1)),
    );

    if !app.save_screen_text_box.content().is_empty() {
        render_widget_clamped(frame, &app.save_button, app.layout.save_game.save_button);
    }

    render_save_file_preview(app, frame);
    render_save_file_metadata(app, frame);
    render_widget_clamped(
        frame,
        &app.previous_screen_button,
        app.layout.previous_screen_button,
    );
}
