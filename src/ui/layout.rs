//! Every width that shapes the layout, in one place — changing a number here changes the screen.

/// Column 1 (groups). Wide enough for `FRAMEWORKS`, narrow enough to leave the topics room.
pub const GROUP_COLUMN: u16 = 22;
/// Column 2 (topics). Fits a monogram, a gap and a name like `Git Version Control`.
pub const TOPICS_COLUMN: u16 = 30;
/// Column 3 (preview) only earns its place when the terminal can pay for it.
pub const PREVIEW_COLUMN: u16 = 40;
pub const PREVIEW_THRESHOLD: u16 = 110;
/// Contents list that lives inside the preview, left of its text.
pub const PREVIEW_CONTENTS: u16 = 16;
/// Single-cell rule between the columns.
pub const GUTTER: u16 = 1;
/// Reading measure. A 200-column line is not read, it is scanned.
pub const READING_WIDTH: u16 = 100;
/// Minimal width the topics column is allowed to keep without a preview.
pub const MIN_TOPICS: u16 = 20;
