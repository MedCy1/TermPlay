use super::{create_note, GameMusic};
use rodio::Sink;

/// Musique électronique rétro pour Pong
pub struct PongMusic;

impl GameMusic for PongMusic {
    fn name(&self) -> &str {
        "Pong Retro Electronic"
    }

    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Mélodie électronique minimaliste inspirée des années 70-80
        // Basée sur des gammes pentatoniques avec un rythme répétitif
        let melody = vec![
            // Intro section - montée
            (330.0, 300), // E4
            (392.0, 300), // G4
            (440.0, 300), // A4
            (523.0, 600), // C5 (accent)
            // Main theme - séquence électronique
            (440.0, 200), // A4
            (440.0, 200), // A4
            (523.0, 400), // C5
            (392.0, 200), // G4
            (330.0, 400), // E4
            (440.0, 200), // A4
            (523.0, 200), // C5
            (659.0, 400), // E5
            (523.0, 200), // C5
            (440.0, 400), // A4
            // Variation - plus aigu
            (523.0, 200), // C5
            (659.0, 200), // E5
            (784.0, 300), // G5
            (880.0, 500), // A5 (climax)
            // Retour au thème principal
            (523.0, 300), // C5
            (440.0, 300), // A4
            (392.0, 300), // G4
            (330.0, 600), // E4 (conclusion)
        ];

        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.7);
            sink.append(note);
        }
    }

    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version accélérée pour les moments intenses (balles rapides)
        let fast_melody = vec![
            // Rythme plus rapide et plus intense
            (440.0, 150), // A4
            (523.0, 150), // C5
            (659.0, 150), // E5
            (784.0, 150), // G5
            (880.0, 300), // A5
            (784.0, 150), // G5
            (659.0, 150), // E5
            (523.0, 150), // C5
            (440.0, 150), // A4
            (392.0, 300), // G4
            // Séquence répétitive rapide
            (523.0, 100), // C5
            (659.0, 100), // E5
            (523.0, 100), // C5
            (659.0, 100), // E5
            (784.0, 400), // G5 (accent)
        ];

        for (freq, duration_ms) in fast_melody {
            let note = create_note(freq, duration_ms, volume * 0.8);
            sink.append(note);
        }
    }

    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Mélodie de victoire - montée triomphante
        let celebration = vec![
            // Gamme ascendante triomphante
            (330.0, 200), // E4
            (392.0, 200), // G4
            (440.0, 200), // A4
            (523.0, 200), // C5
            (659.0, 200), // E5
            (784.0, 300), // G5
            (880.0, 500), // A5 (victoire!)
            // Répétition de la note de victoire avec variations
            (880.0, 200), // A5
            (784.0, 200), // G5
            (880.0, 400), // A5 final
        ];

        for (freq, duration_ms) in celebration {
            let note = create_note(freq, duration_ms, volume * 0.9);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique Pong
pub const PONG_MUSIC: PongMusic = PongMusic;
