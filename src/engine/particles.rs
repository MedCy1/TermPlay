use super::fx::{lerp, put, Rgb};
use rand::Rng;
use ratatui::buffer::Buffer;

const GLYPHS: [char; 5] = ['*', '·', '+', '×', '▀'];

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    glyph: char,
    from: Rgb,
    to: Rgb,
}

/// Pool de particules à capacité fixe (aucune allocation après `new`).
/// Coordonnées en cellules terminal; vitesses en cellules/seconde.
pub struct Particles {
    pool: Vec<Particle>,
    cap: usize,
    pub gravity: f32,
}

impl Particles {
    pub fn new(cap: usize, gravity: f32) -> Self {
        Self {
            pool: Vec::with_capacity(cap),
            cap,
            gravity,
        }
    }

    /// Explosion radiale de `n` particules, couleur `from` → `to` sur leur durée de vie.
    pub fn burst(&mut self, x: f32, y: f32, n: usize, speed: f32, from: Rgb, to: Rgb) {
        self.spray(x, y, n, speed, from, to, 0.0);
    }

    /// Comme `burst`, avec une poussée vers le haut (`lift`, cellules/s).
    #[allow(clippy::too_many_arguments)]
    pub fn spray(&mut self, x: f32, y: f32, n: usize, speed: f32, from: Rgb, to: Rgb, lift: f32) {
        if !super::fx::fx_enabled() {
            return;
        }
        // Effets réduits: un tiers des particules
        let n = if super::fx::fx_low() {
            n.div_ceil(3)
        } else {
            n
        };
        let mut rng = rand::rng();
        for _ in 0..n {
            if self.pool.len() >= self.cap {
                return;
            }
            let ang = rng.random_range(0.0..std::f32::consts::TAU);
            let s = rng.random_range(0.3..1.0) * speed;
            let max_life = rng.random_range(0.4..0.9);
            self.pool.push(Particle {
                x,
                y,
                vx: ang.cos() * s * 2.0, // x doublé: cellules étroites
                vy: ang.sin() * s - lift,
                life: max_life,
                max_life,
                glyph: GLYPHS[rng.random_range(0..GLYPHS.len())],
                from,
                to,
            });
        }
    }

    pub fn update(&mut self, dt: f32) {
        let g = self.gravity;
        self.pool.retain_mut(|p| {
            p.life -= dt;
            p.vy += g * dt;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life > 0.0
        });
    }

    pub fn draw(&self, buf: &mut Buffer, off: (i32, i32)) {
        for p in &self.pool {
            let age = 1.0 - p.life / p.max_life;
            let mut ch = p.glyph;
            if ch == '▀' && p.y.fract().abs() >= 0.5 {
                ch = '▄'; // demi-bloc haut/bas selon la position sous-cellulaire
            }
            put(
                buf,
                p.x.round() as i32 + off.0,
                p.y.floor() as i32 + off.1,
                ch,
                lerp(p.from, p.to, age),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_expires_and_respects_cap() {
        let mut p = Particles::new(8, 10.0);
        p.burst(5.0, 5.0, 100, 5.0, (255, 0, 0), (0, 0, 0));
        assert_eq!(p.pool.len(), 8);
        for _ in 0..100 {
            p.update(0.02);
        }
        assert!(p.pool.is_empty());
    }
}
