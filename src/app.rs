use ratatui::layout::Rect;
use ratatui::widgets::{Gauge, Paragraph};
use strum::VariantArray;

use crate::effects::Effects;
use crate::event::Event;
use crate::input::DoubleClickTracker;
use crate::surface::state::{Seed, SurfaceState};
use crate::surface::{self, Surface};
use crate::ui::documentation::Document;
use crate::ui::main_menu::MainMenu;
use crate::ui::pause_menu::PauseMenu;
use crate::ui::{AppLayout, Screen};
use crate::widgets::button::{BorderAttachedButton, Button, Location, TextButton};
use crate::widgets::list::TextList;
use crate::widgets::optional_list::OptionalTextList;
use crate::widgets::text_box::TextBox;

use std::collections::HashMap;
use std::error;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Local;
use tokio::sync::mpsc::UnboundedSender;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct WorldWindow {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl WorldWindow {
    pub fn zoom_in(&mut self) {
        self.x_min = (self.x_min + 10.0).min(-10.0);
        self.x_max = (self.x_max - 10.0).max(10.0);
        self.y_min = (self.y_min + 10.0).min(-10.0);
        self.y_max = (self.y_max - 10.0).max(10.0);
    }

    pub fn zoom_out(&mut self) {
        self.x_min = (self.x_min - 10.0).max(-200.0);
        self.x_max = (self.x_max + 10.0).min(200.0);
        self.y_min = (self.y_min - 10.0).max(-200.0);
        self.y_max = (self.y_max + 10.0).min(200.0);
    }

    pub fn width(&self) -> u16 {
        (self.x_max - self.x_min) as u16
    }

    pub fn height(&self) -> u16 {
        (self.y_max - self.y_min) as u16
    }
}

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Intermediate {
    Beam,
    Plate,
    Gear,
}

#[derive(Debug)]
pub enum LoadingState {
    Loading,
    Loaded(Box<SurfaceState>),
    Failed(String),
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    pub pause_menu_button: Button<Paragraph<'static>>,
    pub current_research_button: Button<Gauge<'static>>,
    pub tutorial_previous_button: TextButton,
    pub tutorial_next_button: TextButton,
    pub seed: Seed,

    pub prev_tick: Duration,
    pub effects: Effects,

    /// position struct
    // TODO should probably be an option?
    pub surface: Surface,

    /// for loading game
    //pub loading_state: LoadingState,
    pub save_file_cache: HashMap<PathBuf, LoadingState>,

    /// UI
    pub input_mode: InputMode,
    screen: Screen,
    previous_screen: Screen,
    //pub previous_screen_button: PreviousScreenButton,
    pub previous_screen_button: BorderAttachedButton,
    pub copy_button: BorderAttachedButton,
    pub main_menu: TextList<MainMenu>,
    pub documentation: TextList<Document>,
    pub documentation_scroll: u16,
    pub pause_menu: TextList<PauseMenu>,
    pub save_files: OptionalTextList<DisplayPathBuf>,
    pub layout: AppLayout,
    pub tech_tree_double_click_tracker: DoubleClickTracker<usize>,
    pub load_game_double_click_tracker: DoubleClickTracker<u16>,

    pub save_screen_text_box: TextBox,
    pub save_button: BorderAttachedButton,

    pub event_sender: UnboundedSender<Event>,
}

#[derive(Debug, Clone, Copy)]
pub struct ConstructionSite {
    pub area: Rect,
    pub buildable: bool,
}

#[derive(Default, Debug, Clone)]
pub struct DisplayPathBuf {
    pub inner: PathBuf,
}

impl std::fmt::Display for DisplayPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = self
            .inner
            .as_path()
            .file_stem()
            .map(|x| x.to_str().unwrap_or_default())
            .unwrap_or("Error loading filename");
        write!(f, "{name}")
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(event_sender: UnboundedSender<Event>, width: u16, height: u16) -> Self {
        let p = Paragraph::new("Menu [M]").centered();
        let pause_menu_button = Button::new(p);
        let current_research_button = Button::new(Gauge::default());
        let tutorial_previous_button = TextButton::new("<PREV [P]");
        let tutorial_next_button = TextButton::new("[N] NEXT>");
        let copy_button =
            BorderAttachedButton::new(" ⧉  Copy [CTRL + C]".to_string(), Location::East(6));
        let previous_screen_button = BorderAttachedButton::new(
            format!("↻  {} [ESC]", Screen::default()),
            Location::SouthEast,
        );
        let save_button =
            BorderAttachedButton::new("   Save [ENTER]    ".to_string(), Location::East(6));

        let surface = surface::generation::empty(event_sender.clone());

        let list = Document::VARIANTS.to_vec();
        let documentation = TextList::default_style(list);

        let seed = Seed::default();

        //let test_effect = fx::fade_to_fg(Color::Red, (3000, Interpolation::Linear));
        //let test_effect = fx::coalesce(500, (2000, Interpolation::Linear));
        //        let test_effect = fx::sweep_in(
        //            fx::Direction::UpToDown,
        //            30,
        //            Color::Black,
        //            (2000, Interpolation::Linear),
        //        );
        //        let filter: CellFilter =
        //            CellFilter::selector();
        //        let test_effect = fx::slide_in(
        //            fx::Direction::UpToDown,
        //            30,
        //            Color::Black,
        //            (3000, Interpolation::Linear),
        //        ).with_cell_selection(filter)
        //            ;

        let mut app = App {
            running: true,
            seed,
            pause_menu_button,
            current_research_button,
            tutorial_previous_button,
            tutorial_next_button,
            copy_button,
            prev_tick: Duration::ZERO,
            surface,
            //loading_state: LoadingState::Loading,
            save_file_cache: HashMap::default(),
            input_mode: InputMode::Normal,
            main_menu: MainMenu::list(),
            pause_menu: PauseMenu::list(),
            documentation,
            documentation_scroll: 0,
            save_files: OptionalTextList::default(),
            save_screen_text_box: TextBox::default(),
            save_button,
            screen: Screen::default(),
            previous_screen: Screen::default(),
            previous_screen_button,
            layout: AppLayout::default(),
            tech_tree_double_click_tracker: DoubleClickTracker::default(),
            load_game_double_click_tracker: DoubleClickTracker::default(),
            event_sender,
            effects: Effects::new(),
        };
        app.layout = AppLayout::update(width, height, &app);
        app
    }

    //pub fn previous_screen_button() -> Previous

    pub fn loading_state(&self) -> &LoadingState {
        if let Some(path) = self.save_files.selected() {
            self.save_file_cache
                .get(&path.inner)
                .unwrap_or(&LoadingState::Loading)
        } else {
            &LoadingState::Loading
        }
    }

    pub fn fetch_save_files(&mut self) {
        let mut save_files: Vec<DisplayPathBuf> = vec![];
        let save_dir = crate::logging::get_data_dir();
        for entry in save_dir.as_path().read_dir().expect("TODO").flatten() {
            let file = entry.path();
            if file
                .file_name()
                .map(|oss| oss.to_string_lossy().ends_with(".texaform"))
                == Some(true)
            {
                save_files.push(DisplayPathBuf { inner: file })
            }
        }
        let save_files = if save_files.is_empty() {
            OptionalTextList::default_style(save_files)
        } else {
            let mut l = OptionalTextList::default_style(save_files);
            l.select(0);
            l
        };
        self.save_files = save_files;
    }

    pub fn set_screen(&mut self, screen: Screen) {
        self.previous_screen = self.screen;
        self.previous_screen_button
            .update(format!("↻  {} [ESC]", self.previous_screen));
        self.layout.previous_screen_button = self
            .previous_screen_button
            .resize(self.layout.width, self.layout.height);
        self.screen = screen;
        screen.on_load(self)
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn previous_screen(&self) -> &Screen {
        &self.previous_screen
    }

    fn clean_up_autosaves(&mut self) -> Result<(), std::io::Error> {
        let save_path = crate::logging::get_data_dir();
        let mut autosave_paths = vec![];
        for entry in std::fs::read_dir(save_path)? {
            let path = entry?.path();
            let path_is_autosave = path.to_string_lossy().ends_with("_autosave.texaform");
            if path.is_file() && path_is_autosave {
                autosave_paths.push(path);
            }
        }
        if autosave_paths.len() > 10 {
            tracing::info!("over 10 autosaves, attempting to remove oldest autosave file");
            let oldest_file = autosave_paths
                .iter()
                .min_by_key(|path| {
                    if let Ok(metadata) = path.metadata()
                        && let Ok(sys_time) = metadata.modified()
                    {
                        sys_time
                    } else {
                        std::time::SystemTime::now()
                    }
                })
                .expect("min will exist");
            std::fs::remove_file(oldest_file)?;
            match self.save_file_cache.remove(oldest_file) {
                Some(_) => tracing::info!("removed {oldest_file:?} from save_file_cache"),
                None => tracing::info!("{oldest_file:?} not in save_file_cache"),
            }
            tracing::info!("removed {oldest_file:?}");
        }
        Ok(())
    }

    fn autosave_and_cleanup(&mut self) -> AppResult<()> {
        tracing::info!("AUTOSAVE TIME");
        let now = Local::now();
        let formatted_time = now.format("%Y_%m_%d_%H:%M:%S");
        SurfaceState::save(&self.surface, format!("{formatted_time}_autosave"))?;
        match self.clean_up_autosaves() {
            Ok(_) => tracing::info!("cleaned up autosave dir"),
            Err(e) => tracing::error!("failed to clean up autosave dir: {e}"),
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        //TODO run in background
        let five_minutes_of_ticks = 300_000 / crate::TICK_UPDATE_MILLS;
        if (self.surface.game_state.stats.tick_count % five_minutes_of_ticks) == 0
            && !self.surface.agents.is_empty()
            && self.screen != Screen::SaveGame
        {
            match self.autosave_and_cleanup() {
                Ok(_) => (),
                Err(e) => tracing::error!("failed to autosave: {e}"),
            }
        }
        self.surface.tick();
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
