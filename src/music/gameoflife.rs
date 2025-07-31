use super::{GameMusic, create_note, create_chord};
use rodio::Sink;

/// Musique ambiante et évolutive pour Conway's Game of Life
pub struct GameOfLifeMusic;

impl GameMusic for GameOfLifeMusic {
    fn name(&self) -> &str {
        "Game of Life Ambient"
    }
    
    fn play_normal(&self, sink: &Sink, volume: f32) {
        // Musique ambiante contemplative et évolutive, inspirée de la science et l'émerveillement
        // Sons organiques et naturels, comme l'évolution de la vie
        let melody = vec![
            // Intro mystérieuse - émergence de la vie
            (261.3, 800),  // C4 (long, établit l'atmosphère)
            (293.7, 600),  // D4
            (329.6, 400),  // E4
            (261.3, 400),  // C4
            (392.0, 1000), // G4 (soutenu, évocation de l'émerveillement)
            
            // Thème principal - évolution et croissance
            (440.0, 500),  // A4
            (392.0, 400),  // G4
            (349.2, 600),  // F4
            (329.6, 500),  // E4
            (293.7, 400),  // D4
            (329.6, 800),  // E4 (retour harmonieux)
            
            (523.3, 400),  // C5 (montée évolutive)
            (493.9, 300),  // B4
            (440.0, 500),  // A4
            (392.0, 600),  // G4
            (349.2, 400),  // F4
            (392.0, 1000), // G4 (stabilisation)
            
            // Variation - complexité émergente
            (659.3, 600),  // E5 (nouvelle génération)
            (587.3, 400),  // D5
            (523.3, 500),  // C5
            (440.0, 400),  // A4
            (493.9, 600),  // B4
            (440.0, 800),  // A4 (harmonie)
            
            // Conclusion - cycle de vie
            (392.0, 500),  // G4
            (329.6, 400),  // E4
            (293.7, 600),  // D4
            (261.3, 1200), // C4 (finale contemplative)
        ];
        
        for (freq, duration_ms) in melody {
            let note = create_note(freq, duration_ms, volume * 0.4); // Volume très doux
            sink.append(note);
        }
    }
    
    fn play_fast(&self, sink: &Sink, volume: f32) {
        // Version plus dynamique pour les simulations rapides
        // Rythme accéléré mais toujours contemplatif
        let dynamic_melody = vec![
            // Rythme plus soutenu, évolution rapide
            (261.3, 300),  // C4
            (329.6, 250),  // E4
            (392.0, 300),  // G4
            (523.3, 400),  // C5
            (440.0, 350),  // A4
            
            (493.9, 250),  // B4
            (523.3, 300),  // C5
            (659.3, 400),  // E5
            (587.3, 300),  // D5
            (523.3, 500),  // C5
            
            // Complexité croissante
            (784.0, 300),  // G5
            (659.3, 250),  // E5
            (587.3, 200),  // D5
            (523.3, 250),  // C5
            (440.0, 400),  // A4
            (493.9, 350),  // B4
            
            // Patterns évolutifs
            (523.3, 200),  // C5
            (659.3, 200),  // E5
            (784.0, 250),  // G5
            (880.0, 300),  // A5
            (784.0, 400),  // G5
            
            // Résolution harmonieuse
            (659.3, 350),  // E5
            (523.3, 300),  // C5
            (440.0, 400),  // A4
            (392.0, 600),  // G4 (conclusion)
        ];
        
        for (freq, duration_ms) in dynamic_melody {
            let note = create_note(freq, duration_ms, volume * 0.5);
            sink.append(note);
        }
    }
    
    fn play_celebration(&self, sink: &Sink, volume: f32) {
        // Musique d'émerveillement - pour les patterns complexes stables
        // Plus épique et émotionnelle, célébrant la beauté des automates cellulaires
        let wonder_sequence = vec![
            // Ouverture majestueuse - révélation de la beauté
            (261.3, 400),  // C4
            (329.6, 400),  // E4
            (392.0, 400),  // G4
            (523.3, 600),  // C5
            (659.3, 800),  // E5 (émerveillement)
            
            // Mélodie épique de découverte
            (784.0, 500),  // G5
            (880.0, 400),  // A5
            (1046.5, 600), // C6 (climax)
            (880.0, 400),  // A5
            (784.0, 500),  // G5
            (659.3, 600),  // E5
            
            // Harmonie complexe (simulation d'accords)
            (523.3, 400),  // C5
            (659.3, 400),  // E5 (conceptuellement avec C5)
            (784.0, 400),  // G5
            (1046.5, 800), // C6 (accord majestueux)
            
            // Évolution mélodique - patterns qui émergent
            (880.0, 300),  // A5
            (784.0, 300),  // G5
            (659.3, 300),  // E5
            (587.3, 300),  // D5
            (659.3, 400),  // E5
            (784.0, 600),  // G5 (stabilisation)
            
            // Finale cosmique
            (1046.5, 400), // C6
            (1174.7, 300), // D6
            (1318.5, 400), // E6
            (1046.5, 600), // C6
            (784.0, 500),  // G5
            (523.3, 1000), // C5 (finale contemplative)
        ];
        
        for (freq, duration_ms) in wonder_sequence {
            let note = create_note(freq, duration_ms, volume * 0.6);
            sink.append(note);
        }
    }
}

/// Instance globale de la musique Game of Life
pub const GAMEOFLIFE_MUSIC: GameOfLifeMusic = GameOfLifeMusic;