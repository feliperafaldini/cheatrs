pub mod data;
pub mod model;
pub mod search;
pub mod theme;
pub mod ui;

pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Below this the layout stops being a layout; the app refuses instead of degrading.
pub fn fits(cols: u16, rows: u16) -> bool {
    cols >= MIN_WIDTH && rows >= MIN_HEIGHT
}

/// The one sentence for a terminal that is too small, whether at boot or mid-session.
pub fn size_warning(cols: u16, rows: u16) -> String {
    format!("cheatrs needs at least {MIN_WIDTH}x{MIN_HEIGHT} (terminal is {cols}x{rows})")
}
