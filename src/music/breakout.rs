use super::{GameMusic, create_note, create_chord};
use rodio::Sink;

/// Musique arcade énergique pour Breakout
pub struct BreakoutMusic;

impl GameMusic for BreakoutMusic {
    fn name(&self) -> &str {
        "Breakout Arcade"
    }
    
    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Mélodie arcade énergique et entraînante, style années 80
        let melody = vec![
            // Intro énergique - progression majeure
            (523.3, 300),  // C5
            (587.3, 300),  // D5
            (783.9, 400),  // G5
            (523.3, 200),  // C5
            (659.3, 600),  // E5
            
            // Thème principal - rythme soutenu
            (784.0, 200),  // G5
            (659.3, 200),  // E5
            (587.3, 200),  // D5
            (523.3, 400),  // C5
            (440.0, 300),  // A4
            (523.3, 500),  // C5
            
            (659.3, 300),  // E5
            (784.0, 300),  // G5
            (880.0, 400),  // A5
            (784.0, 200),  // G5
            (659.3, 600),  // E5
            
            // Variation montante - énergie croissante
            (523.3, 200),  // C5
            (659.3, 200),  // E5
            (784.0, 200),  // G5
            (880.0, 300),  // A5
            (1046.5, 400), // C6 (climax)
            (880.0, 300),  // A5
            (784.0, 500),  // G5
            
            // Conclusion entraînante
            (659.3, 300),  // E5
            (784.0, 300),  // G5
            (523.3, 300),  // C5
            (659.3, 400),  // E5
            (523.3, 800),  // C5 (finale)
        ];
        
        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.6);
            sink.append(note);
        }
    }
    
    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version plus rapide et intense pour les moments critiques (peu de briques restantes)
        let intense_melody = vec![
            // Rythme accéléré, notes plus courtes
            (1046.5, 150), // C6
            (880.0, 150),  // A5
            (1046.5, 150), // C6
            (1174.7, 200), // D6
            (1318.5, 300), // E6
            
            (1046.5, 150), // C6
            (880.0, 150),  // A5
            (784.0, 150),  // G5
            (880.0, 200),  // A5
            (1046.5, 400), // C6
            
            // Séquence répétitive intense
            (880.0, 100),  // A5
            (1046.5, 100), // C6
            (1318.5, 100), // E6
            (1568.0, 150), // G6
            (1318.5, 200), // E6
            (1046.5, 300), // C6
            
            // Montée dramatique
            (784.0, 100),  // G5
            (880.0, 100),  // A5
            (1046.5, 100), // C6
            (1318.5, 100), // E6
            (1568.0, 400), // G6 (tension maximale)
            
            // Résolution énergique
            (1318.5, 200), // E6
            (1046.5, 200), // C6
            (880.0, 200),  // A5
            (1046.5, 600), // C6 (finale intense)
        ];
        
        for (freq, duration_ms) in intense_melody {
            let note = create_note(freq, duration_ms, volume * 0.7);
            sink.append(note);
        }
    }
    
    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Musique de victoire arcade - fanfare triomphante
        let victory_sequence = vec![
            // Fanfare d'ouverture
            (523.3, 200),  // C5
            (659.3, 200),  // E5
            (784.0, 200),  // G5
            (1046.5, 300), // C6
            (1318.5, 400), // E6
            
            // Mélodie triomphante
            (1046.5, 300), // C6
            (1174.7, 300), // D6
            (1318.5, 400), // E6
            (1046.5, 200), // C6
            (880.0, 200),  // A5
            (1046.5, 500), // C6
            
            // Gamme ascendante victorieuse
            (784.0, 150),  // G5
            (880.0, 150),  // A5
            (1046.5, 150), // C6
            (1174.7, 150), // D6
            (1318.5, 150), // E6
            (1479.1, 150), // F#6
            (1568.0, 300), // G6
            (1568.0, 600), // G6 (soutenu)
            
            // Accord final triomphant (simulation)
            (523.3, 400),  // C5
            (659.3, 400),  // E5 (conceptuellement avec C5)
            (784.0, 400),  // G5
            (1046.5, 600), // C6 (finale épique)
            
            // Echo de victoire
            (784.0, 300),  // G5
            (1046.5, 300), // C6
            (1318.5, 600), // E6 (finale glorieuse)
        ];
        
        for (freq, duration_ms) in victory_sequence {
            let note = create_note(freq, duration_ms, volume * 0.8);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique Breakout
pub const BREAKOUT_MUSIC: BreakoutMusic = BreakoutMusic;