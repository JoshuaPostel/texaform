use ratatui::layout::{Margin, Rect};

use ratatui::style::{Color, Modifier};

use crate::draw::{PubCell, SetCell};

pub struct BorderedRectangle {
    pub left: char,
    pub right: char,
    pub top: char,
    pub bot: char,
    pub top_left: char,
    pub top_right: char,
    pub bot_left: char,
    pub bot_right: char,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
    pub inner_cell: PubCell,
}

pub fn draw_rectangle<T: SetCell>(area: &Rect, cell: PubCell, settable: &mut T) {
    for x in area.x..area.x + area.width {
        for y in area.y..area.y + area.height {
            settable.set_cell(x, y, cell);
        }
    }
}

pub fn draw_bordered_rectangle<T: SetCell>(
    area: &Rect,
    border: &BorderedRectangle,
    settable: &mut T,
) {
    // TODO if rect height <=2 and width <= 2 inner_cell gets drawn at 0,0
    let rect = area.inner(Margin {
        horizontal: 1,
        vertical: 1,
    });
    draw_rectangle(&rect, border.inner_cell, settable);

    let y_max = area.y + area.height - 1;
    let x_max = area.x + area.width - 1;

    for y in area.y..y_max {
        let pub_cell = PubCell {
            c: border.left,
            fg: border.fg,
            bg: border.bg,
            modifier: border.modifier,
        };
        settable.set_cell(area.x, y, pub_cell);

        let pub_cell = PubCell {
            c: border.right,
            fg: border.fg,
            bg: border.bg,
            modifier: border.modifier,
        };
        settable.set_cell(x_max, y, pub_cell);
    }

    for x in area.x..(area.x + area.width) {
        let pub_cell = PubCell {
            c: border.top,
            fg: border.fg,
            bg: border.bg,
            modifier: border.modifier,
        };
        settable.set_cell(x, area.y, pub_cell);

        let pub_cell = PubCell {
            c: border.bot,
            fg: border.fg,
            bg: border.bg,
            modifier: border.modifier,
        };
        settable.set_cell(x, y_max, pub_cell);
    }

    let pub_cell = PubCell {
        c: border.top_left,
        fg: border.fg,
        bg: border.bg,
        modifier: border.modifier,
    };
    settable.set_cell(area.x, area.y, pub_cell);

    let pub_cell = PubCell {
        c: border.top_right,
        fg: border.fg,
        bg: border.bg,
        modifier: border.modifier,
    };
    settable.set_cell(x_max, area.y, pub_cell);

    let pub_cell = PubCell {
        c: border.bot_left,
        fg: border.fg,
        bg: border.bg,
        modifier: border.modifier,
    };
    settable.set_cell(area.x, y_max, pub_cell);

    let pub_cell = PubCell {
        c: border.bot_right,
        fg: border.fg,
        bg: border.bg,
        modifier: border.modifier,
    };
    settable.set_cell(x_max, y_max, pub_cell);
}

//    let border = BorderedRectangle {
//        top_left: '▛',
//        top: '▀',
//        top_right: '▜',
//        left: '▌',
//        right: '▐',
//        bot_left: '▙',
//        bot: '▄',
//        bot_right: '▟',
//        fg: Color::DarkGray,
//        bg: Color::Black,
//        modifier: Modifier::empty(),
//        inner_cell,
//    };

//    let border = BorderedRectangle {
//        top_left: '┏',
//        top: '━',
//        top_right: '┓',
//        left: '┃',
//        right: '┃',
//        bot_left: '┗',
//        bot: '━',
//        bot_right: '┛',
//        fg: Color::Yellow,
//        bg: Color::Black,
//        modifier: Modifier::empty(),
//        inner_cell,
//    };

//    let border = BorderedRectangle {
//        top_left: '╔',
//        top: '═',
//        top_right: '╗',
//        left: '║',
//        right: '║',
//        bot_left: '╚',
//        bot: '═',
//        bot_right: '╝',
//        fg: Color::Yellow,
//        bg: Color::Black,
//        modifier: Modifier::empty(),
//        inner_cell,
//    };

//    let border = BorderedRectangle {
//        top_left: '╔',
//        top: '╦',
//        top_right: '╗',
//        left: '╠',
//        right: '╣',
//        bot_left: '╚',
//        bot: '╩',
//        bot_right: '╝',
//        fg: Color::Yellow,
//        bg: Color::Black,
//        modifier: Modifier::empty(),
//        inner_cell,
//    };

pub fn draw_construction<T: SetCell>(area: &Rect, settable: &mut T) {
    let inner_cell = PubCell {
        c: '╱',
        fg: Color::Yellow,
        bg: Color::Black,
        modifier: Modifier::BOLD,
    };

    let border = BorderedRectangle {
        left: '▐',
        right: '▌',
        top: '▄',
        bot: '▀',
        top_left: '▗',
        top_right: '▖',
        bot_left: '▝',
        bot_right: '▘',
        fg: Color::Yellow,
        bg: Color::DarkGray,
        modifier: Modifier::empty(),
        inner_cell,
    };

    draw_bordered_rectangle(area, &border, settable)
}
