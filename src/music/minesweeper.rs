use super::{create_note, GameMusic};
use rodio::Sink;

/// Musique tendue et contemplative pour Minesweeper
pub struct MinesweeperMusic;

impl GameMusic for MinesweeperMusic {
    fn name(&self) -> &str {
        "Minesweeper Tension"
    }

    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Musique contemplative et tendue, mélodie mineure mystérieuse
        let melody = vec![
            // Intro mystérieuse - tons mineurs
            (220.0, 800), // A3 (long, établit la tension)
            (246.9, 400), // B3
            (261.3, 600), // C4
            (293.7, 400), // D4
            (246.9, 800), // B3 (retour, tension)
            // Thème principal - progression inquiétante
            (329.6, 500), // E4
            (293.7, 300), // D4
            (261.3, 500), // C4
            (246.9, 600), // B3
            (220.0, 400), // A3
            (261.3, 400), // C4
            (293.7, 400), // D4
            (329.6, 300), // E4
            (349.2, 500), // F4 (moment de tension)
            (329.6, 800), // E4 (résolution partielle)
            // Variation - montée progressive de tension
            (220.0, 300), // A3
            (261.3, 300), // C4
            (293.7, 300), // D4
            (349.2, 400), // F4
            (392.0, 600), // G4 (climax de tension)
            // Retour au thème avec variations
            (329.6, 500),  // E4
            (293.7, 400),  // D4
            (246.9, 600),  // B3
            (220.0, 1000), // A3 (finale suspendue)
        ];

        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.4); // Volume plus bas pour la concentration
            sink.append(note);
        }
    }

    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version plus rapide et tendue pour les moments critiques
        let intense_melody = vec![
            // Rythme plus soutenu, notes plus courtes
            (220.0, 300), // A3
            (246.9, 200), // B3
            (261.3, 300), // C4
            (293.7, 200), // D4
            (329.6, 400), // E4
            (349.2, 250), // F4
            (392.0, 250), // G4
            (440.0, 300), // A4 (montée d'octave)
            (392.0, 200), // G4
            (349.2, 400), // F4
            // Séquence répétitive stressante
            (329.6, 200), // E4
            (293.7, 200), // D4
            (261.3, 200), // C4
            (246.9, 200), // B3
            (220.0, 300), // A3
            (261.3, 150), // C4
            (329.6, 150), // E4
            (392.0, 150), // G4
            (440.0, 500), // A4 (tension maximale)
            // Conclusion tendue
            (392.0, 300), // G4
            (349.2, 300), // F4
            (329.6, 300), // E4
            (293.7, 600), // D4 (résolution partielle)
        ];

        for (freq, duration_ms) in intense_melody {
            let note = create_note(freq, duration_ms, volume * 0.5);
            sink.append(note);
        }
    }

    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Musique de victoire - libération de la tension
        let victory_sequence = vec![
            // Gamme ascendante libératrice
            (261.3, 200), // C4
            (293.7, 200), // D4
            (329.6, 200), // E4
            (349.2, 200), // F4
            (392.0, 200), // G4
            (440.0, 200), // A4
            (493.9, 200), // B4
            (523.3, 400), // C5 (victoire!)
            // Mélodie triomphante majeure
            (523.3, 300), // C5
            (587.3, 300), // D5
            (659.3, 400), // E5
            (523.3, 300), // C5
            (440.0, 300), // A4
            (523.3, 500), // C5
            // Fanfare finale
            (659.3, 400), // E5
            (523.3, 200), // C5
            (659.3, 200), // E5
            (784.0, 300), // G5
            (523.3, 600), // C5 (final triomphant)
            // Echo de victoire
            (392.0, 300), // G4
            (523.3, 300), // C5
            (659.3, 600), // E5 (finale épique)
        ];

        for (freq, duration_ms) in victory_sequence {
            let note = create_note(freq, duration_ms, volume * 0.7);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique Minesweeper
pub const MINESWEEPER_MUSIC: MinesweeperMusic = MinesweeperMusic;
