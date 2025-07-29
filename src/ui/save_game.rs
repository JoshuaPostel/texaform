use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::app::{App, InputMode};
use crate::ui::load_game::{render_save_file_metadata, render_save_file_preview};
use crate::ui::render_widget_clamped;

#[derive(Debug, Default)]
pub struct SaveGameLayout {
    pub save_files: Rect,
    pub save_file_input: Rect,
    pub save_button: Rect,
    pub surface_preview: Rect,
    pub metadata: Rect,
}

impl SaveGameLayout {
    pub fn new(width: u16, height: u16) -> SaveGameLayout {
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
            //Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]).split(columns[1]);
            Layout::vertical([Constraint::Max(3),Constraint::Fill(1)]).split(columns[1]);

        let input_area =
            Layout::horizontal([Constraint::Fill(1), Constraint::Min(10)]).split(right_column[0]);

        SaveGameLayout {
            save_files: right_column[1],
            save_file_input: input_area[0],
            save_button: input_area[1],
            surface_preview: left_column[0],
            metadata: left_column[1],
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let border = Block::bordered()
        .title("Save Files")
        .style(Style::new().bg(Color::Black).fg(Color::Green));
    render_widget_clamped(frame, border, app.layout.save_game.save_files);

    render_widget_clamped(
        frame,
        app.save_files.clone(),
        app.layout.save_game.save_files.inner(Margin::new(1, 1)),
    );

    let append = match app.input_mode {
        InputMode::Normal => "",
        InputMode::Editing => "█",
    };

    let text_input =
        Paragraph::new("> ".to_string() + app.save_screen_text_box.input.as_str() + append)
            .block(Block::bordered().title("Name"))
            .fg(Color::Green)
            .bg(Color::Black);

    render_widget_clamped(frame, text_input, app.layout.save_game.save_file_input);

    let label = if app
        .save_files
        .items
        .iter()
        .any(|i| i.to_string() == app.save_screen_text_box.input)
    {
        "Overwrite [↵ ENTER]"
    } else {
        "Save [↵ ENTER]"
    };
    render_widget_clamped(
        frame,
        app.save_button.with_content(Paragraph::new(label)),
        app.layout.save_game.save_button,
    );

    render_save_file_preview(app, frame);
    render_save_file_metadata(app, frame);
    render_widget_clamped(
        frame,
        app.previous_screen_button.clone(),
        app.layout.previous_screen_button,
    );
}
