pub mod tetris;
pub mod snake;
pub mod pong;

use rodio::{source::{Source, SineWave}, Sink};
use std::time::Duration;

/// Trait pour définir les différentes musiques du jeu
pub trait GameMusic {
    /// Joue la musique normale
    fn play_normal(&self, sink: &Sink, volume: f32);
    
    /// Joue la musique rapide (pour les niveaux élevés)
    fn play_fast(&self, sink: &Sink, volume: f32);
    
    /// Joue la musique de célébration
    fn play_celebration(&self, sink: &Sink, volume: f32);
    
    /// Nom de la musique
    fn name(&self) -> &str;
}

/// Helper pour créer des notes avec fade in/out - Compatible Rodio 0.21
pub fn create_note(frequency: f32, duration_ms: u64, volume: f32) -> Box<dyn Source<Item = f32> + Send> {
    // Dans Rodio 0.21, frequency doit être > 0
    let safe_freq = if frequency <= 0.0 { 1.0 } else { frequency };
    let safe_volume = if frequency <= 0.0 { 0.0 } else { volume };
    
    Box::new(
        SineWave::new(safe_freq)
            .take_duration(Duration::from_millis(duration_ms))
            .fade_in(Duration::from_millis(10.min(duration_ms / 4)))
            .fade_out(Duration::from_millis(30.min(duration_ms / 3)))
            .amplify(safe_volume)
    )
}

/// Helper pour créer des accords (plusieurs notes simultanées) - Compatible Rodio 0.21
pub fn create_chord(frequencies: &[f32], duration_ms: u64, volume: f32) -> Box<dyn Source<Item = f32> + Send> {
    if frequencies.is_empty() {
        return create_note(1.0, duration_ms, 0.0); // Fréquence 1Hz inaudible au lieu de 0.0
    }
    
    if frequencies.len() == 1 {
        return create_note(frequencies[0], duration_ms, volume);
    }
    
    // Pour simplifier avec Rodio 0.21, on joue juste la première note de l'accord
    // TODO: Implémenter un vrai système d'accords plus tard
    Box::new(
        SineWave::new(frequencies[0])
            .take_duration(Duration::from_millis(duration_ms))
            .fade_in(Duration::from_millis(20.min(duration_ms / 4)))
            .fade_out(Duration::from_millis(50.min(duration_ms / 3)))
            .amplify(volume)
    )
}