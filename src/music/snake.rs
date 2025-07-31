use super::{create_note, GameMusic};
use rodio::Sink;

/// Musique simple et répétitive pour Snake
pub struct SnakeMusic;

impl GameMusic for SnakeMusic {
    fn name(&self) -> &str {
        "Snake Ambient"
    }

    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Mélodie simple et apaisante pour Snake
        // Basée sur une progression d'accords mineure
        let melody = vec![
            (440.0, 600), // A4
            (523.0, 400), // C5
            (659.0, 600), // E5
            (587.0, 400), // D5
            (523.0, 600), // C5
            (440.0, 400), // A4
            (392.0, 800), // G4 (plus longue)
            // Variation
            (523.0, 600), // C5
            (659.0, 400), // E5
            (784.0, 600), // G5
            (659.0, 400), // E5
            (523.0, 600), // C5
            (440.0, 400), // A4
            (392.0, 800), // G4
        ];

        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.6);
            sink.append(note);
        }
    }

    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version plus rapide avec des notes plus courtes
        let fast_melody = vec![
            (440.0, 300), // A4
            (523.0, 200), // C5
            (659.0, 300), // E5
            (587.0, 200), // D5
            (523.0, 300), // C5
            (440.0, 200), // A4
            (392.0, 400), // G4
            (523.0, 300), // C5
            (659.0, 200), // E5
            (784.0, 300), // G5
            (659.0, 200), // E5
            (523.0, 300), // C5
            (440.0, 200), // A4
            (392.0, 400), // G4
        ];

        for (freq, duration_ms) in fast_melody {
            let note = create_note(freq, duration_ms, volume * 0.7);
            sink.append(note);
        }
    }

    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Petite mélodie de célébration quand le serpent mange
        let celebration = vec![
            (659.0, 150),  // E5
            (784.0, 150),  // G5
            (880.0, 150),  // A5
            (1046.0, 300), // C6 (plus aigu)
        ];

        for (freq, duration_ms) in celebration {
            let note = create_note(freq, duration_ms, volume * 0.8);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique Snake
pub const SNAKE_MUSIC: SnakeMusic = SnakeMusic;
