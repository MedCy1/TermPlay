use super::{GameMusic, create_note, create_chord};
use rodio::Sink;

/// Musique de Tetris (Korobeiniki)
pub struct TetrisMusic;

impl GameMusic for TetrisMusic {
    fn name(&self) -> &str {
        "Tetris (Korobeiniki)"
    }
    
    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Mélodie principale de Korobeiniki
        // E B C D C B A A C E D C B C D E C A A
        let main_melody = vec![
            (659.0, 400), // E5
            (493.0, 200), // B4
            (523.0, 200), // C5
            (587.0, 400), // D5
            (523.0, 200), // C5
            (493.0, 200), // B4
            (440.0, 400), // A4
            (440.0, 200), // A4
            (523.0, 200), // C5
            (659.0, 400), // E5
            (587.0, 200), // D5
            (523.0, 200), // C5
            (493.0, 600), // B4 (plus long)
            (523.0, 200), // C5
            (587.0, 400), // D5
            (659.0, 400), // E5
            (523.0, 400), // C5
            (440.0, 400), // A4
            (440.0, 400), // A4
            
            // Deuxième partie: D F A G F E C E D C B B C D E C A A
            (587.0, 600), // D5 (plus long)
            (698.0, 200), // F5
            (880.0, 400), // A5
            (784.0, 200), // G5
            (698.0, 200), // F5
            (659.0, 600), // E5 (plus long)
            (523.0, 200), // C5
            (659.0, 400), // E5
            (587.0, 200), // D5
            (523.0, 200), // C5
            (493.0, 400), // B4
            (493.0, 200), // B4
            (523.0, 200), // C5
            (587.0, 400), // D5
            (659.0, 400), // E5
            (523.0, 400), // C5
            (440.0, 400), // A4
            (440.0, 400), // A4
        ];
        
        // Ligne de basse simple pour accompagnement
        let bass_notes = vec![
            (329.0, 800),  // E3
            (220.0, 800),  // A3
            (207.0, 800),  // Ab3
            (329.0, 800),  // E3
            (220.0, 800),  // A3
            (293.0, 800),  // D3
            (261.0, 800),  // C3
            (329.0, 800),  // E3
        ];
        
        // Jouer la mélodie principale
        for (freq, duration_ms) in main_melody {
            let note = create_note(freq, duration_ms, volume * 0.8);
            sink.append(note);
        }
        
        // Ajouter quelques notes de basse en arrière-plan (plus doucement)
        for (freq, duration_ms) in bass_notes.iter().take(4) {
            let bass_note = create_note(*freq, *duration_ms, volume * 0.3);
            sink.append(bass_note);
        }
    }
    
    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version accélérée - notes plus courtes
        let fast_melody = vec![
            (659.0, 200), // E5
            (493.0, 100), // B4
            (523.0, 100), // C5
            (587.0, 200), // D5
            (523.0, 100), // C5
            (493.0, 100), // B4
            (440.0, 200), // A4
            (440.0, 100), // A4
            (523.0, 100), // C5
            (659.0, 200), // E5
            (587.0, 100), // D5
            (523.0, 100), // C5
            (493.0, 300), // B4
            (523.0, 100), // C5
            (587.0, 200), // D5
            (659.0, 200), // E5
            (523.0, 200), // C5
            (440.0, 200), // A4
            (440.0, 200), // A4
        ];
        
        for (freq, duration_ms) in fast_melody {
            let note = create_note(freq, duration_ms, volume);
            sink.append(note);
        }
    }
    
    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Version avec harmonies pour célébrer un Tetris!
        let celebration_chords = vec![
            // Accord de victoire: E + G + C
            (&[659.0, 784.0, 523.0][..], 400), // E5 + G5 + C5
            (&[659.0, 784.0, 523.0][..], 400), // Répétition
            (&[659.0, 784.0, 523.0][..], 600), // Plus long pour la finale
        ];
        
        for (frequencies, duration_ms) in celebration_chords {
            let chord = create_chord(frequencies, duration_ms, volume * 1.2);
            sink.append(chord);
        }
    }
}

/// Instance globale de la musique Tetris
pub const TETRIS_MUSIC: TetrisMusic = TetrisMusic;