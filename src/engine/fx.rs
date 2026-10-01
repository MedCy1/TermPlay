use rand::Rng;
use ratatui::{buffer::Buffer, layout::Rect, style::Color, style::Style};

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub type Rgb = (u8, u8, u8);

/// Capacité couleur du terminal, résolue une fois au démarrage (`init`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    True,
    Palette256,
    Mono,
}

static MODE: AtomicU8 = AtomicU8::new(0);
static FX_ON: AtomicBool = AtomicBool::new(true);

/// Pure (testable): (mode, effets actifs) selon flags et environnement.
pub fn detect(
    no_fx: bool,
    no_color: bool,
    no_color_env: Option<&str>,
    colorterm: Option<&str>,
    windows: bool,
) -> (ColorMode, bool) {
    let mono = no_color || no_color_env.is_some_and(|v| !v.is_empty());
    let truecolor = colorterm.is_some_and(|v| v == "truecolor" || v == "24bit");
    // ponytail: Windows Terminal ne définit pas COLORTERM mais gère le 24 bits
    let mode = if mono {
        ColorMode::Mono
    } else if truecolor || windows {
        ColorMode::True
    } else {
        ColorMode::Palette256
    };
    (mode, !no_fx && mode != ColorMode::Mono)
}

pub fn init(no_fx: bool, no_color: bool) {
    let (mode, fx) = detect(
        no_fx,
        no_color,
        std::env::var("NO_COLOR").ok().as_deref(),
        std::env::var("COLORTERM").ok().as_deref(),
        cfg!(windows),
    );
    MODE.store(mode as u8, Ordering::Relaxed);
    FX_ON.store(fx, Ordering::Relaxed);
}

pub fn color_mode() -> ColorMode {
    match MODE.load(Ordering::Relaxed) {
        0 => ColorMode::True,
        1 => ColorMode::Palette256,
        _ => ColorMode::Mono,
    }
}

/// Particules, shake, fondus.
pub fn fx_enabled() -> bool {
    FX_ON.load(Ordering::Relaxed)
}

/// Lueurs: demandent du 24 bits (les dégradés subtils bandent en 256 couleurs).
fn glow_enabled() -> bool {
    fx_enabled() && color_mode() == ColorMode::True
}

/// Couleur RGB → indice xterm-256 le plus proche (cube 6×6×6 ou rampe de gris).
pub fn to_256((r, g, b): Rgb) -> u8 {
    const L: [i32; 6] = [0, 95, 135, 175, 215, 255];
    let near = |v: u8| (0..6).min_by_key(|&i| (L[i] - v as i32).abs()).unwrap_or(0);
    let (qr, qg, qb) = (near(r), near(g), near(b));
    let dist = |a: (i32, i32, i32)| {
        (a.0 - r as i32).pow(2) + (a.1 - g as i32).pow(2) + (a.2 - b as i32).pow(2)
    };
    let cube_d = dist((L[qr], L[qg], L[qb]));
    let avg = (r as i32 + g as i32 + b as i32) / 3;
    let gi = ((avg - 8 + 5) / 10).clamp(0, 23);
    let gray = 8 + 10 * gi;
    if dist((gray, gray, gray)) < cube_d {
        232 + gi as u8
    } else {
        (16 + 36 * qr + 6 * qg + qb) as u8
    }
}

/// À appeler sur le buffer fini de chaque frame: adapte les couleurs au terminal.
pub fn finish(buf: &mut Buffer) {
    let mode = color_mode();
    if mode == ColorMode::True {
        return;
    }
    let conv = |c: Color| match (mode, c) {
        (ColorMode::Mono, _) => Color::Reset,
        (_, Color::Rgb(r, g, b)) => Color::Indexed(to_256((r, g, b))),
        _ => c,
    };
    for c in &mut buf.content {
        c.fg = conv(c.fg);
        c.bg = conv(c.bg);
    }
}

pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

pub fn color(c: Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

/// Approximation RGB (palette xterm) des couleurs nommées; `Reset` → `reset`.
fn rgb_of(c: Color, reset: Rgb) -> Rgb {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Reset => reset,
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::Gray => (229, 229, 229),
        Color::DarkGray => (102, 102, 102),
        Color::LightRed => (241, 76, 76),
        Color::LightGreen => (35, 209, 139),
        Color::LightYellow => (245, 245, 67),
        Color::LightBlue => (59, 142, 234),
        Color::LightMagenta => (214, 112, 214),
        Color::LightCyan => (41, 184, 219),
        Color::White => (255, 255, 255),
        _ => reset, // ponytail: couleurs indexées 0-255 traitées comme `reset`
    }
}

/// Accès à une cellule avec coordonnées signées, `None` hors du buffer (safe avec le shake).
pub fn cell(buf: &mut Buffer, x: i32, y: i32) -> Option<&mut ratatui::buffer::Cell> {
    if x < 0 || y < 0 || x > u16::MAX as i32 || y > u16::MAX as i32 {
        return None;
    }
    buf.cell_mut((x as u16, y as u16))
}

pub fn put(buf: &mut Buffer, x: i32, y: i32, ch: char, fg: Rgb) {
    if let Some(c) = cell(buf, x, y) {
        c.set_char(ch).set_fg(color(fg));
    }
}

/// Comme `put`, avec fond et gras optionnel.
pub fn put_bg(buf: &mut Buffer, x: i32, y: i32, ch: char, fg: Rgb, bg: Rgb, bold: bool) {
    if let Some(c) = cell(buf, x, y) {
        c.set_char(ch).set_fg(color(fg)).set_bg(color(bg));
        if bold {
            c.modifier.insert(ratatui::style::Modifier::BOLD);
        }
    }
}

/// Lueur: éclaircit le fond autour de (cx, cy). Cellules ~2x plus hautes que larges → x compressé.
pub fn glow(buf: &mut Buffer, cx: i32, cy: i32, radius: i32, c: Rgb, intensity: f32) {
    if !glow_enabled() {
        return;
    }
    let r = radius as f32;
    for dy in -radius..=radius {
        for dx in -2 * radius..=2 * radius {
            let d = ((dx as f32 / 2.0).powi(2) + (dy * dy) as f32).sqrt();
            if d >= r {
                continue;
            }
            let k = (1.0 - d / r).powi(2) * intensity;
            if let Some(cell) = cell(buf, cx + dx, cy + dy) {
                cell.set_bg(color(lerp(rgb_of(cell.bg, (0, 0, 0)), c, k)));
            }
        }
    }
}

/// Fondu vers le noir (t: 0 = rien, 1 = noir) sur toute une zone.
pub fn fade_to_black(buf: &mut Buffer, area: Rect, t: f32) {
    if !fx_enabled() {
        return;
    }
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                let fg = lerp(rgb_of(c.fg, (204, 204, 204)), (0, 0, 0), t);
                let bg = lerp(rgb_of(c.bg, (0, 0, 0)), (0, 0, 0), t);
                c.set_style(Style::new().fg(color(fg)).bg(color(bg)));
            }
        }
    }
}

/// Screen shake à décroissance exponentielle.
#[derive(Default)]
pub struct Shake {
    amp: f32,
}

impl Shake {
    pub fn kick(&mut self, amp: f32) {
        if !fx_enabled() {
            return;
        }
        self.amp = self.amp.max(amp);
    }

    pub fn update(&mut self, dt: f32) {
        self.amp *= (-9.0 * dt).exp();
        if self.amp < 0.15 {
            self.amp = 0.0;
        }
    }

    /// Offset en cellules terminal (x doublé: les cellules sont étroites).
    pub fn offset(&self) -> (i32, i32) {
        if self.amp == 0.0 {
            return (0, 0);
        }
        let mut rng = rand::rng();
        let a = self.amp;
        (
            (rng.random_range(-a..=a) * 2.0).round() as i32,
            rng.random_range(-a..=a).round() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_color_support() {
        use ColorMode::*;
        assert_eq!(
            detect(false, false, None, Some("truecolor"), false),
            (True, true)
        );
        assert_eq!(
            detect(false, false, None, Some("24bit"), false),
            (True, true)
        );
        assert_eq!(detect(false, false, None, None, false), (Palette256, true));
        assert_eq!(detect(false, false, None, None, true), (True, true));
        assert_eq!(
            detect(false, false, Some("1"), Some("truecolor"), false),
            (Mono, false)
        );
        assert_eq!(
            detect(false, false, Some(""), Some("truecolor"), false),
            (True, true)
        ); // NO_COLOR vide ignoré
        assert_eq!(
            detect(false, true, None, Some("truecolor"), false),
            (Mono, false)
        );
        assert_eq!(
            detect(true, false, None, Some("truecolor"), false),
            (True, false)
        );
    }

    #[test]
    fn quantizes_to_xterm_256() {
        assert_eq!(to_256((0, 0, 0)), 16);
        assert_eq!(to_256((255, 255, 255)), 231);
        assert_eq!(to_256((255, 0, 0)), 196);
        assert_eq!(to_256((128, 128, 128)), 244); // rampe de gris
    }

    #[test]
    fn lerp_and_shake() {
        assert_eq!(lerp((0, 0, 0), (200, 100, 50), 0.5), (100, 50, 25));
        assert_eq!(lerp((0, 0, 0), (9, 9, 9), 2.0), (9, 9, 9));
        let mut s = Shake::default();
        s.kick(2.0);
        for _ in 0..60 {
            s.update(0.016);
        }
        assert_eq!(s.offset(), (0, 0));
    }
}
