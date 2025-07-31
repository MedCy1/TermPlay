use super::{GameMusic, create_note, create_chord};
use rodio::Sink;

/// Musique relaxante et moderne pour 2048
pub struct Game2048Music;

impl GameMusic for Game2048Music {
    fn name(&self) -> &str {
        "2048 Zen Mode"
    }
    
    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Mélodie zen et contemplative, inspirée de la musique de puzzle moderne
        // Utilise des progressions d'accords apaisantes
        let melody = vec![
            // Intro - progression douce
            (261.3, 600),  // C4
            (293.7, 400),  // D4
            (329.6, 600),  // E4
            (349.2, 800),  // F4 (plus long)
            
            // Thème principal - mélodie fluide
            (392.0, 400),  // G4
            (349.2, 400),  // F4
            (329.6, 400),  // E4
            (293.7, 600),  // D4
            (261.3, 600),  // C4
            
            (329.6, 400),  // E4
            (392.0, 400),  // G4
            (440.0, 600),  // A4
            (392.0, 400),  // G4
            (349.2, 800),  // F4
            
            // Variation - montée progressive
            (440.0, 300),  // A4
            (493.9, 300),  // B4
            (523.3, 600),  // C5
            (493.9, 400),  // B4
            (440.0, 400),  // A4
            (392.0, 600),  // G4
            
            // Conclusion apaisante
            (349.2, 600),  // F4
            (329.6, 600),  // E4
            (293.7, 600),  // D4
            (261.3, 1000), // C4 (finale)
        ];
        
        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.6);
            sink.append(note);
        }
    }
    
    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version plus énergique pour les moments de combo/points élevés
        let energetic_melody = vec![
            // Rythme plus soutenu, notes plus courtes
            (523.3, 200),  // C5
            (587.3, 200),  // D5
            (659.3, 200),  // E5
            (698.5, 300),  // F5
            (784.0, 400),  // G5
            
            (659.3, 200),  // E5
            (698.5, 200),  // F5
            (784.0, 200),  // G5
            (880.0, 300),  // A5
            (784.0, 400),  // G5
            
            // Séquence répétitive énergique
            (523.3, 150),  // C5
            (659.3, 150),  // E5
            (784.0, 150),  // G5
            (880.0, 150),  // A5
            (1046.5, 600), // C6 (climax)
            
            // Retour progressif
            (880.0, 300),  // A5
            (784.0, 300),  // G5
            (659.3, 300),  // E5
            (523.3, 600),  // C5
        ];
        
        for (freq, duration_ms) in energetic_melody {
            let note = create_note(freq, duration_ms, volume * 0.7);
            sink.append(note);
        }
    }
    
    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Mélodie de victoire pour atteindre 2048 ou plus
        let victory_sequence = vec![
            // Gamme ascendante triomphante
            (261.3, 150),  // C4
            (293.7, 150),  // D4
            (329.6, 150),  // E4
            (349.2, 150),  // F4
            (392.0, 150),  // G4
            (440.0, 150),  // A4
            (493.9, 150),  // B4
            (523.3, 300),  // C5
            
            // Accord de victoire répété
            (523.3, 400),  // C5
            (659.3, 400),  // E5 (joué conceptuellement avec C5)
            (784.0, 400),  // G5
            (1046.5, 600), // C6 (finale triomphante)
            
            // Echo de la victoire
            (523.3, 300),  // C5
            (659.3, 300),  // E5
            (784.0, 600),  // G5 final
        ];
        
        for (freq, duration_ms) in victory_sequence {
            let note = create_note(freq, duration_ms, volume * 0.8);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique 2048
pub const GAME2048_MUSIC: Game2048Music = Game2048Music;