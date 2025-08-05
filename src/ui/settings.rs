use ratatui::{
    Frame,
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use crate::app::App;
use crate::ui::render_widget_clamped;

//use tokio::time::{sleep, Duration};

pub fn render(app: &App, frame: &mut Frame) {
    let block = Paragraph::new("TODO")
        .block(Block::bordered().title("Settings"))
        .style(Style::default().fg(Color::Green).bg(Color::Black));

    render_widget_clamped(frame, block, app.layout.whole_screen());
    render_widget_clamped(
        frame,
        app.previous_screen_button.clone(),
        app.layout.previous_screen_button,
    );
}
