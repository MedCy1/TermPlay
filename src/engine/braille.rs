use super::fx::{cell, color, lerp, Rgb};
use ratatui::buffer::Buffer;

/// Bit Braille pour chaque sous-pixel, indexé [colonne][ligne].
const BITS: [[u8; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
const LIT: u8 = 90; // intensité minimale pour allumer un point

/// Canvas Braille: 2×4 sous-pixels par cellule, intensité u8 par point (permet les traînées).
/// Coordonnées en cellules (f32); une cellule couvre [n, n+1).
pub struct Braille {
    w: usize,
    h: usize,
    dots: Vec<u8>,
}

impl Braille {
    pub fn new(w: u16, h: u16) -> Self {
        let mut b = Self {
            w: 0,
            h: 0,
            dots: Vec::new(),
        };
        b.resize(w, h);
        b
    }

    /// Ne réalloue que si la taille change.
    pub fn resize(&mut self, w: u16, h: u16) {
        let (w, h) = (w as usize, h as usize);
        if (w, h) != (self.w, self.h) {
            self.w = w;
            self.h = h;
            self.dots.clear();
            self.dots.resize(w * 2 * h * 4, 0);
        }
    }

    /// Disque de rayon `r` (en sous-pixels) centré en (x, y).
    pub fn blob(&mut self, x: f32, y: f32, r: f32) {
        let (cx, cy) = (x * 2.0, y * 4.0);
        let (dw, dh) = (self.w * 2, self.h * 4);
        let x0 = ((cx - r).floor().max(0.0)) as usize;
        let x1 = ((cx + r).ceil().max(0.0) as usize).min(dw.saturating_sub(1));
        let y0 = ((cy - r).floor().max(0.0)) as usize;
        let y1 = ((cy + r).ceil().max(0.0) as usize).min(dh.saturating_sub(1));
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (ddx, ddy) = (px as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                if ddx * ddx + ddy * ddy <= r * r {
                    self.dots[py * dw + px] = 255;
                }
            }
        }
    }

    /// Segment continu (balle rapide sans trous): empreintes espacées de ~0.7 sous-pixel.
    pub fn line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, r: f32) {
        let len = ((x1 - x0) * 2.0).hypot((y1 - y0) * 4.0);
        let n = (len / 0.7).ceil().max(1.0) as usize;
        for i in 0..=n {
            let t = i as f32 / n as f32;
            self.blob(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, r);
        }
    }

    /// Décroissance exponentielle (constante de temps `tau` secondes) → traînée.
    pub fn decay(&mut self, dt: f32, tau: f32) {
        let k = (-dt / tau).exp();
        for d in &mut self.dots {
            *d = (*d as f32 * k) as u8;
        }
    }

    /// Écrit les cellules non vides dans `buf` à partir de (ox, oy). Teinte: `dim` → `ink` selon l'intensité.
    pub fn draw(&self, buf: &mut Buffer, ox: i32, oy: i32, ink: Rgb, dim: Rgb) {
        let dw = self.w * 2;
        for cy in 0..self.h {
            for cx in 0..self.w {
                let (mut mask, mut peak) = (0u8, 0u8);
                for (sx, col) in BITS.iter().enumerate() {
                    for (sy, bit) in col.iter().enumerate() {
                        let v = self.dots[(cy * 4 + sy) * dw + cx * 2 + sx];
                        if v >= LIT {
                            mask |= bit;
                            peak = peak.max(v);
                        }
                    }
                }
                if mask != 0 {
                    if let Some(c) = cell(buf, ox + cx as i32, oy + cy as i32) {
                        let ch = char::from_u32(0x2800 + mask as u32).unwrap_or(' ');
                        c.set_char(ch)
                            .set_fg(color(lerp(dim, ink, peak as f32 / 255.0)));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn blob_draws_and_decays() {
        let mut b = Braille::new(4, 2);
        b.blob(1.5, 0.5, 1.5);
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
        b.draw(&mut buf, 0, 0, (255, 255, 255), (0, 0, 0));
        assert!(buf.content().iter().any(|c| c.symbol() != " "));
        b.decay(1.0, 0.05);
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
        b.draw(&mut buf, 0, 0, (255, 255, 255), (0, 0, 0));
        assert!(buf.content().iter().all(|c| c.symbol() == " "));
        b.blob(-5.0, 99.0, 2.0); // hors canvas: pas de panic
    }
}
