use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use ratatui::{
    backend::Backend,
    crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use std::{
    io::{self, stdout},
    panic::{set_hook, take_hook},
};
use texaform::app::{App, AppResult, InputMode};
use texaform::event::{Event, EventHandler};
use texaform::logging::initialize_logging;
use texaform::tui::Tui;
use texaform::ui::AppLayout;

use texaform::input::{handle_key_events, handle_mouse_events};

use texaform::TICK_UPDATE_MILLS;

#[tokio::main]
async fn main() -> AppResult<()> {
    init_panic_hook();
    initialize_logging()?;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    // just over 60 fps
    // tick() to understand how many ticks per second
    let events = EventHandler::new(TICK_UPDATE_MILLS);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    // initalize ui locations
    let event_sender = tui.events.sender.clone();

    // Create an application.
    let (width, height) = crossterm::terminal::size()?;
    let mut app = App::new(event_sender.clone(), width, height);

    let mut last_frame_instant = std::time::Instant::now();
    // Start the main loop.
    while app.running {
        app.prev_tick = last_frame_instant.elapsed();
        last_frame_instant = std::time::Instant::now();
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(event) => handle_key_events(event, &mut app).await?,
            Event::Mouse(event) => handle_mouse_events(event, &mut app).await?,
            Event::Resize(w, h) => {
                app.layout = AppLayout::update(w, h, &app);
            }
            // TODO seprate thread and update loop for agents?
            // currntly it would tui.draw for each event from each agent
            Event::AgentConnection(port, address) => {
                if let Some(comms) = app.surface.agents.get_mut(&port) {
                    comms.address = Some(address);
                    app.input_mode = InputMode::Normal;
                } else {
                    tracing::warn!("expected agent at port {port}");
                }
            }
            Event::AgentDisconect(port) => {
                if let Some(comms) = app.surface.agents.get_mut(&port) {
                    comms.address = None;
                } else {
                    tracing::warn!("expected agent at port {port}");
                }
            }
            Event::AgentCommand(port, command) => {
                app.surface.update_agent_remote(&port, command).await;
            }
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}

// coppied from https://ratatui.rs/recipes/apps/panic-hooks/
fn init_panic_hook() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

pub fn init_tui() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
