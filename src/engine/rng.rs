//! Aléa de *gameplay* (pièces, nourriture, mines, balle, IA). Si `TERMPLAY_SEED` est défini,
//! le flux est déterministe: mêmes pièces et même nourriture à chaque lancement, ce qui rend
//! les démos VHS reproductibles. Les effets purement visuels (particules, shake) gardent
//! l'aléa normal, pour ne pas décaler le gameplay selon le timing.
use rand::{rngs::StdRng, rngs::ThreadRng, RngCore, SeedableRng};
use std::sync::Mutex;

struct Source(Mutex<Option<StdRng>>);

impl Source {
    const fn new() -> Self {
        Self(Mutex::new(None))
    }

    fn seed(&self, seed: Option<u64>) {
        *self.0.lock().unwrap_or_else(|e| e.into_inner()) = seed.map(StdRng::seed_from_u64);
    }

    /// `None` si non initialisé (aléa normal).
    fn with<T>(&self, f: impl FnOnce(&mut StdRng) -> T) -> Option<T> {
        let mut g = self.0.lock().unwrap_or_else(|e| e.into_inner());
        g.as_mut().map(f)
    }
}

static SOURCE: Source = Source::new();

/// Lit `TERMPLAY_SEED` (entier). Valeur absente ou invalide: aléa normal.
pub fn init() {
    SOURCE.seed(parse_seed(std::env::var("TERMPLAY_SEED").ok().as_deref()));
}

fn parse_seed(v: Option<&str>) -> Option<u64> {
    v.and_then(|s| s.trim().parse().ok())
}

/// Générateur de gameplay: flux seedé partagé, ou `ThreadRng` sans graine.
pub struct GameRng(ThreadRng);

pub fn game_rng() -> GameRng {
    GameRng(rand::rng())
}

impl RngCore for GameRng {
    fn next_u32(&mut self) -> u32 {
        SOURCE
            .with(|r| r.next_u32())
            .unwrap_or_else(|| self.0.next_u32())
    }
    fn next_u64(&mut self) -> u64 {
        SOURCE
            .with(|r| r.next_u64())
            .unwrap_or_else(|| self.0.next_u64())
    }
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        if SOURCE.with(|r| r.fill_bytes(dst)).is_none() {
            self.0.fill_bytes(dst)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn same_seed_same_sequence() {
        let s = Source::new();
        let draw =
            |s: &Source| s.with(|r| (0..8).map(|_| r.random_range(0..7)).collect::<Vec<u8>>());
        assert_eq!(draw(&s), None); // sans graine: aléa normal
        s.seed(Some(42));
        let a = draw(&s);
        s.seed(Some(42));
        assert_eq!(a, draw(&s));
        s.seed(Some(43));
        assert_ne!(a, draw(&s));
    }

    #[test]
    fn parses_seed() {
        assert_eq!(parse_seed(Some(" 7 ")), Some(7));
        assert_eq!(parse_seed(Some("abc")), None);
        assert_eq!(parse_seed(None), None);
    }
}
