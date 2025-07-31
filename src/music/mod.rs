pub mod tetris;
pub mod snake;

use rodio::{source::Source, Sink};
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

/// Helper pour créer des notes avec fade in/out
pub fn create_note(frequency: f32, duration_ms: u64, volume: f32) -> Box<dyn Source<Item = f32> + Send> {
    use rodio::source::SineWave;
    
    Box::new(
        SineWave::new(frequency)
            .take_duration(Duration::from_millis(duration_ms))
            .fade_in(Duration::from_millis(10.min(duration_ms / 4)))
            .fade_out(Duration::from_millis(30.min(duration_ms / 3)))
            .amplify(volume)
    )
}

/// Helper pour créer des accords (plusieurs notes simultanées)
pub fn create_chord(frequencies: &[f32], duration_ms: u64, volume: f32) -> Box<dyn Source<Item = f32> + Send> {
    use rodio::source::SineWave;
    
    if frequencies.is_empty() {
        return create_note(0.0, duration_ms, 0.0);
    }
    
    if frequencies.len() == 1 {
        return create_note(frequencies[0], duration_ms, volume);
    }
    
    // Créer la première note
    let mut chord_source: Box<dyn Source<Item = f32> + Send> = Box::new(
        SineWave::new(frequencies[0])
            .take_duration(Duration::from_millis(duration_ms))
    );
    
    // Ajouter les autres notes
    for &freq in &frequencies[1..] {
        let note = SineWave::new(freq)
            .take_duration(Duration::from_millis(duration_ms));
        chord_source = Box::new(chord_source.mix(note));
    }
    
    Box::new(
        chord_source
            .fade_in(Duration::from_millis(20.min(duration_ms / 4)))
            .fade_out(Duration::from_millis(50.min(duration_ms / 3)))
            .amplify(volume)
    )
}