use std::sync::OnceLock;

use ratatui::style::Color;

/// How much colour the terminal can be trusted with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth {
    /// `NO_COLOR` or `TERM=dumb`: hierarchy lives on bold, dim and rules only.
    Mono,
    /// Sixteen ANSI colours: everything is mapped to the nearest one.
    Ansi16,
    /// 256-colour palette.
    Ansi256,
    /// Truecolor.
    True,
}

const BG: Color = Color::Rgb(0x0f, 0x12, 0x14);
const SURFACE: Color = Color::Rgb(0x17, 0x1b, 0x1e);
const RULE: Color = Color::Rgb(0x26, 0x2c, 0x31);
const FG: Color = Color::Rgb(0xd6, 0xdb, 0xdf);
const FG_DIM: Color = Color::Rgb(0x8b, 0x94, 0x9a);
const FG_FAINT: Color = Color::Rgb(0x5c, 0x66, 0x6d);

/// Minimum contrast for anything that carries content.
const MIN_CONTRAST: f32 = 4.5;

/// The sixteen ANSI colours as they are usually rendered, for nearest-colour matching.
const ANSI16: [(Color, (u8, u8, u8)); 16] = [
    (Color::Black, (0, 0, 0)),
    (Color::Red, (0x80, 0x00, 0x00)),
    (Color::Green, (0x00, 0x80, 0x00)),
    (Color::Yellow, (0x80, 0x80, 0x00)),
    (Color::Blue, (0x00, 0x00, 0x80)),
    (Color::Magenta, (0x80, 0x00, 0x80)),
    (Color::Cyan, (0x00, 0x80, 0x80)),
    (Color::Gray, (0xc0, 0xc0, 0xc0)),
    (Color::DarkGray, (0x80, 0x80, 0x80)),
    (Color::LightRed, (0xff, 0x00, 0x00)),
    (Color::LightGreen, (0x00, 0xff, 0x00)),
    (Color::LightYellow, (0xff, 0xff, 0x00)),
    (Color::LightBlue, (0x00, 0x00, 0xff)),
    (Color::LightMagenta, (0xff, 0x00, 0xff)),
    (Color::LightCyan, (0x00, 0xff, 0xff)),
    (Color::White, (0xff, 0xff, 0xff)),
];

static DEPTH: OnceLock<Depth> = OnceLock::new();

/// Read the environment once; every later call is free.
pub fn depth() -> Depth {
    *DEPTH.get_or_init(detect)
}

fn detect() -> Depth {
    if std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()) {
        return Depth::Mono;
    }
    let term = std::env::var("TERM").unwrap_or_default();
    let colour_term = std::env::var("COLORTERM").unwrap_or_default();

    if term == "dumb" {
        Depth::Mono
    } else if colour_term == "truecolor" || colour_term == "24bit" {
        Depth::True
    } else if term.contains("256color") {
        Depth::Ansi256
    } else if term.is_empty() {
        // No TERM at all is not evidence of a weak terminal; assume a modern one.
        Depth::True
    } else {
        Depth::Ansi16
    }
}

pub fn bg() -> Color {
    paint(BG)
}

pub fn surface() -> Color {
    paint(SURFACE)
}

pub fn rule() -> Color {
    paint(RULE)
}

pub fn fg() -> Color {
    paint(FG)
}

pub fn fg_dim() -> Color {
    paint(FG_DIM)
}

pub fn fg_faint() -> Color {
    paint(FG_FAINT)
}

/// Reduce a colour to what this terminal can actually show.
pub fn paint(color: Color) -> Color {
    paint_with(depth(), color)
}

pub fn paint_with(depth: Depth, color: Color) -> Color {
    match depth {
        Depth::Mono => Color::Reset,
        Depth::True => color,
        Depth::Ansi256 => match rgb_of(color) {
            Some((r, g, b)) => Color::Indexed(nearest_256(r, g, b)),
            None => Color::Reset,
        },
        Depth::Ansi16 => match rgb_of(color) {
            Some((r, g, b)) => nearest_16(r, g, b),
            None => Color::Reset,
        },
    }
}

/// Syntax colours arrive as raw RGB from the highlighter; they pass the same gate.
pub fn paint_rgb(r: u8, g: u8, b: u8) -> Color {
    paint(Color::Rgb(r, g, b))
}

fn rgb_of(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

fn distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> u32 {
    let dr = i32::from(a.0) - i32::from(b.0);
    let dg = i32::from(a.1) - i32::from(b.1);
    let db = i32::from(a.2) - i32::from(b.2);
    (dr * dr + dg * dg + db * db) as u32
}

fn nearest_16(r: u8, g: u8, b: u8) -> Color {
    ANSI16
        .iter()
        .min_by_key(|(_, rgb)| distance((r, g, b), *rgb))
        .map(|(colour, _)| *colour)
        .unwrap_or(Color::Reset)
}

/// The 6×6×6 cube plus the grey ramp, which is what xterm calls 16..=255.
fn nearest_256(r: u8, g: u8, b: u8) -> u8 {
    let steps = [0u8, 95, 135, 175, 215, 255];
    let mut best = (u32::MAX, 16u8);
    for (i, &sr) in steps.iter().enumerate() {
        for (j, &sg) in steps.iter().enumerate() {
            for (k, &sb) in steps.iter().enumerate() {
                let index = 16 + (36 * i + 6 * j + k) as u8;
                let d = distance((r, g, b), (sr, sg, sb));
                if d < best.0 {
                    best = (d, index);
                }
            }
        }
    }
    for step in 0..24u8 {
        let level = 8 + step * 10;
        let d = distance((r, g, b), (level, level, level));
        if d < best.0 {
            best = (d, 232 + step);
        }
    }
    best.1
}

fn parse_hex(hex: &str) -> Option<(f32, f32, f32)> {
    let h = hex.trim().trim_start_matches('#');
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()? as f32 / 255.0;
    let g = u8::from_str_radix(&h[2..4], 16).ok()? as f32 / 255.0;
    let b = u8::from_str_radix(&h[4..6], 16).ok()? as f32 / 255.0;
    Some((r, g, b))
}

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f32::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / d + 2.0) / 6.0
    } else {
        ((r - g) / d + 4.0) / 6.0
    };
    (h, s, l)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s <= f32::EPSILON {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn luminance(r: f32, g: f32, b: f32) -> f32 {
    let f = |c: f32| {
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b)
}

fn contrast(a: (f32, f32, f32), b: (f32, f32, f32)) -> f32 {
    let la = luminance(a.0, a.1, a.2);
    let lb = luminance(b.0, b.1, b.2);
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn bg_rgb() -> (f32, f32, f32) {
    match BG {
        Color::Rgb(r, g, b) => (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        _ => (0.0, 0.0, 0.0),
    }
}

/// The topic colour as ink: hue kept, saturation capped, lightness pinned to a legible band.
/// A colour that still fails 4.5:1 falls back to dim text — the topic loses hue, not legibility.
pub fn accent(hex: &str) -> Color {
    accent_at(depth(), hex)
}

pub fn accent_at(depth: Depth, hex: &str) -> Color {
    let Some((r, g, b)) = parse_hex(hex) else {
        return paint_with(depth, FG_DIM);
    };
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let tuned = hsl_to_rgb(h, s.min(0.62), l.clamp(0.62, 0.74));
    if contrast(tuned, bg_rgb()) < MIN_CONTRAST {
        return paint_with(depth, FG_DIM);
    }
    paint_with(
        depth,
        Color::Rgb(
            (tuned.0 * 255.0).round() as u8,
            (tuned.1 * 255.0).round() as u8,
            (tuned.2 * 255.0).round() as u8,
        ),
    )
}
