use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use crate::app::{App, LoadingState};
use crate::ui::render_widget_clamped;

use crate::TICK_UPDATE_MILLS;
use crate::utils::human_readable_tick_count;

#[derive(Debug, Default)]
pub struct LoadGameLayout {
    pub save_files: Rect,
    pub surface_preview: Rect,
    pub metadata: Rect,
}

impl LoadGameLayout {
    pub fn new(width: u16, height: u16) -> LoadGameLayout {
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

        LoadGameLayout {
            //TODO surface::render_grid() only works in top left
            // save_files: chunks[0],
            // surface_preview: chunks[1],
            save_files: columns[1],
            surface_preview: left_column[0],
            metadata: left_column[1],
        }
    }
}

pub fn render(app: &App, frame: &mut Frame) {
    let border = Block::bordered()
        .title("Save Files")
        .style(Style::new().bg(Color::Black).fg(Color::Green));
    render_widget_clamped(frame, border, app.layout.load_game.save_files);

    render_widget_clamped(
        frame,
        app.save_files.clone(),
        app.layout.load_game.save_files.inner(Margin::new(1, 1)),
    );

    render_save_file_preview(app, frame);
    render_save_file_metadata(app, frame);
    render_widget_clamped(
        frame,
        app.previous_screen_button.clone(),
        app.layout.previous_screen_button,
    );
}

pub fn render_save_file_preview(app: &App, frame: &mut Frame) {
    let paragraph = Paragraph::new("")
        .block(Block::bordered().title("Preview"))
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, paragraph, app.layout.load_game.surface_preview);

    match app.loading_state() {
        LoadingState::Loaded(state) => state.grid.render_preview(
            frame,
            app.layout
                .load_game
                .surface_preview
                .inner(Margin::new(1, 1)),
        ),
        LoadingState::Loading => {
            let paragraph = Paragraph::new("loading...")
                .style(Style::default().fg(Color::Green).bg(Color::Black));
            render_widget_clamped(
                frame,
                paragraph,
                app.layout
                    .load_game
                    .surface_preview
                    .inner(Margin::new(1, 1)),
            );
        }
        LoadingState::Failed(error) => {
            let paragraph = Paragraph::new(format!("failed to load:\n{error}"))
                .style(Style::default().fg(Color::Green).bg(Color::Black));
            render_widget_clamped(
                frame,
                paragraph,
                app.layout
                    .load_game
                    .surface_preview
                    .inner(Margin::new(1, 1)),
            );
        }
    }
}

pub fn render_save_file_metadata(app: &App, frame: &mut Frame) {
    let content = match app.loading_state() {
        LoadingState::Loaded(state) => {
            let playtime = human_readable_tick_count(state.game_stats.tick_count);
            let tech_count = state.game_state.tech_tree.graph.raw_nodes().iter().count();
            let unlocked = state.game_state.tech_tree.unlocked_count();
            format!("playtime: {playtime}\ntechnology: {unlocked}/{tech_count}")
        }
        LoadingState::Loading => "loading...".to_string(),
        LoadingState::Failed(error) => {
            format!("failed to load:\n{error}")
        }
    };

    let paragraph = Paragraph::new(content)
        .block(Block::bordered().title("Metadata"))
        .style(Style::default().fg(Color::Green).bg(Color::Black));
    render_widget_clamped(frame, paragraph, app.layout.load_game.metadata);
}
