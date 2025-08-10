use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Stylize},
    widgets::{Block, Paragraph},
};

use crate::app::{App, InputMode};
use crate::ui::load_game::{
    render_save_file_metadata, render_save_file_preview, save_file_boarder,
};
use crate::ui::{render_widget_clamped, render_widget_ref_clamped};
use crate::widgets::button::{BorderAttachedButton, Location};

#[derive(Debug, Default)]
pub struct SaveGameLayout {
    pub save_files: Rect,
    pub save_file_input: Rect,
    pub surface_preview: Rect,
    pub metadata: Rect,
    pub save_button: Rect,
}

impl SaveGameLayout {
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
        app.save_files.clone(),
        app.layout.save_game.save_files.inner(Margin::new(1, 1)),
    );

    let text_input = Paragraph::new(format!("> {}█", app.save_screen_text_box.input))
        .block(Block::bordered().title("Name"))
        .fg(Color::Green)
        .bg(Color::Black);

    render_widget_clamped(frame, text_input, app.layout.save_game.save_file_input);
    render_widget_ref_clamped(frame, &app.save_button, app.layout.save_game.save_button);

    render_save_file_preview(app, frame);
    render_save_file_metadata(app, frame);
    render_widget_clamped(
        frame,
        &app.previous_screen_button,
        app.layout.previous_screen_button,
    );
}
