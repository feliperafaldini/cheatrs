use ratatui::style::Color;

pub const BG: Color = Color::Rgb(0x0f, 0x12, 0x14);
pub const SURFACE: Color = Color::Rgb(0x17, 0x1b, 0x1e);
pub const RULE: Color = Color::Rgb(0x26, 0x2c, 0x31);
pub const FG: Color = Color::Rgb(0xd6, 0xdb, 0xdf);
pub const FG_DIM: Color = Color::Rgb(0x8b, 0x94, 0x9a);
pub const FG_FAINT: Color = Color::Rgb(0x5c, 0x66, 0x6d);

/// Minimum contrast for anything that carries content.
const MIN_CONTRAST: f32 = 4.5;

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
    let Some((r, g, b)) = parse_hex(hex) else {
        return FG_DIM;
    };
    let (h, s, l) = rgb_to_hsl(r, g, b);
    let tuned = hsl_to_rgb(h, s.min(0.62), l.clamp(0.62, 0.74));
    if contrast(tuned, bg_rgb()) < MIN_CONTRAST {
        return FG_DIM;
    }
    Color::Rgb(
        (tuned.0 * 255.0).round() as u8,
        (tuned.1 * 255.0).round() as u8,
        (tuned.2 * 255.0).round() as u8,
    )
}
