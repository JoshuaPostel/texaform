use ratatui::style::Color;
use tachyonfx::{Effect, Interpolation, fx};

pub struct Effects {
    pub main_menu_logo: Effect,
    pub load_document: Effect,
}

//    let foo = CellFilter::position_fn(move |pos| !app.layout.main_menu.menu.contains(pos));
//    let effect = fx::coalesce(500, (5000, Interpolation::Linear))
//    .with_cell_selection(foo);

impl Effects {
    pub fn new() -> Effects {
        Effects {
            main_menu_logo: fx::coalesce(500, (5000, Interpolation::Linear)),
            load_document: fx::slide_in(
                fx::Direction::UpToDown,
                20,
                Color::Black,
                (500, Interpolation::Linear),
            ),
        }
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}
