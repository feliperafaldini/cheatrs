//! Every width that shapes the layout, in one place — changing a number here changes the screen.
//!
//! The two left columns are measured from the library at startup, not fixed: the
//! widest group label and the widest topic title set them, inside these bounds.
//! Everything saved there goes to the column that holds the actual text.

/// Column 1 (groups): measured from the longest label, clamped here.
pub const GROUP_COLUMN_MIN: u16 = 12;
pub const GROUP_COLUMN_MAX: u16 = 20;
/// Column 2 (topics): bar, monogram and title, clamped here.
pub const TOPICS_COLUMN_MIN: u16 = 22;
pub const TOPICS_COLUMN_MAX: u16 = 36;
/// Column 3 (preview) only earns its place when the terminal can pay for it.
pub const PREVIEW_COLUMN: u16 = 40;
pub const PREVIEW_THRESHOLD: u16 = 110;
/// The contents list inside the preview takes three tenths of it, within bounds.
pub const PREVIEW_CONTENTS_SHARE: u16 = 3;
pub const PREVIEW_CONTENTS_MIN: u16 = 14;
pub const PREVIEW_CONTENTS_MAX: u16 = 22;
/// Single-cell rule between the columns.
pub const GUTTER: u16 = 1;
/// Reading measure. A 200-column line is not read, it is scanned.
pub const READING_WIDTH: u16 = 100;
/// Minimal width the topics column is allowed to keep without a preview.
pub const MIN_TOPICS: u16 = 20;
