use rodio::{
    source::{SineWave, Source, SquareWave},
    OutputStream, OutputStreamBuilder, Sink,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub enum SoundEffect {
    // Snake
    SnakeEat,
    SnakeGameOver,
    
    // Tetris
    TetrisLineClear,
    TetrisPieceDrop,
    TetrisGameOver,
    TetrisRotate,
    TetrisMove,
    TetrisHardDrop,
    TetrisTetris, // 4 lignes d'un coup
    
    // Pong
    PongPaddleHit,
    PongWallHit,
    PongScore,
    
    // Breakout
    BreakoutPaddleHit,
    BreakoutBrickHit,
    BreakoutGameOver,
    
    // 2048
    Game2048Move,
    Game2048Merge,
    Game2048GameOver,
    Game2048Victory,
    
    // Game of Life
    GameOfLifeStep,
    
    // UI
    MenuSelect,
    MenuConfirm,
}

// Notes musicales en Hz (pour référence future)
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Note {
    C4 = 261,
    Cs4 = 277,
    D4 = 293,
    Ds4 = 311,
    E4 = 329,
    F4 = 349,
    Fs4 = 370,
    G4 = 392,
    Gs4 = 415,
    A4 = 440,
    As4 = 466,
    B4 = 493,
    C5 = 523,
    Cs5 = 554,
    D5 = 587,
    Ds5 = 622,
    E5 = 659,
    F5 = 698,
    Fs5 = 740,
    G5 = 784,
    Gs5 = 831,
    A5 = 880,
    As5 = 932,
    B5 = 987,
    Rest = 0,
}

pub struct AudioManager {
    _stream: Option<OutputStream>,
    effects_sink: Option<Sink>,
    music_sink: Option<Sink>,
    volume: Arc<Mutex<f32>>,
    music_volume: Arc<Mutex<f32>>,
    enabled: Arc<Mutex<bool>>,
    music_enabled: Arc<Mutex<bool>>,
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream_handle) => {
                // Créer deux sinks : un pour les effets, un pour la musique
                let effects_sink = Sink::connect_new(&stream_handle.mixer());
                let music_sink = Sink::connect_new(&stream_handle.mixer());
                
                Ok(Self {
                    _stream: Some(stream_handle),
                    effects_sink: Some(effects_sink),
                    music_sink: Some(music_sink),
                    volume: Arc::new(Mutex::new(0.7)),
                    music_volume: Arc::new(Mutex::new(0.3)),
                    enabled: Arc::new(Mutex::new(true)),
                    music_enabled: Arc::new(Mutex::new(true)),
                })
            }
            Err(e) => {
                eprintln!("Erreur d'initialisation audio: {}", e);
                // Fallback en cas d'échec d'initialisation audio
                Ok(Self {
                    _stream: None,
                    effects_sink: None,
                    music_sink: None,
                    volume: Arc::new(Mutex::new(0.7)),
                    music_volume: Arc::new(Mutex::new(0.3)),
                    enabled: Arc::new(Mutex::new(false)),
                    music_enabled: Arc::new(Mutex::new(false)),
                })
            }
        }
    }
    
    pub fn play_sound(&self, effect: SoundEffect) {
        if !*self.enabled.lock().unwrap() {
            return;
        }
        
        if let Some(sink) = &self.effects_sink {
            let volume = *self.volume.lock().unwrap();
            let source = self.generate_sound(effect);
            
            if let Some(source) = source {
                // Volume spécial pour certains effets
                let final_volume = match effect {
                    SoundEffect::TetrisGameOver | 
                    SoundEffect::SnakeGameOver | 
                    SoundEffect::BreakoutGameOver | 
                    SoundEffect::Game2048GameOver => volume.max(0.4),
                    SoundEffect::TetrisTetris => volume * 1.2, // Plus fort pour Tetris!
                    _ => volume,
                };
                
                sink.append(source.amplify(final_volume));
            }
        }
    }
    
    fn generate_sound(&self, effect: SoundEffect) -> Option<Box<dyn Source<Item = f32> + Send>> {
        match effect {
            // Snake sounds
            SoundEffect::SnakeEat => {
                Some(Box::new(
                    SineWave::new(800.0)
                        .take_duration(Duration::from_millis(100))
                ))
            }
            SoundEffect::SnakeGameOver => {
                Some(Box::new(
                    SquareWave::new(200.0)
                        .take_duration(Duration::from_millis(500))
                ))
            }
            
            // Tetris sounds
            SoundEffect::TetrisLineClear => {
                // Son harmonieux pour ligne complétée
                Some(Box::new(
                    SineWave::new(659.0) // E5
                        .mix(SineWave::new(523.0)) // C5
                        .take_duration(Duration::from_millis(300))
                ))
            }
            SoundEffect::TetrisPieceDrop => {
                // Son mat pour pièce posée
                Some(Box::new(
                    SquareWave::new(220.0)
                        .take_duration(Duration::from_millis(80))
                ))
            }
            SoundEffect::TetrisRotate => {
                // Son aigu pour rotation
                Some(Box::new(
                    SineWave::new(880.0) // A5
                        .take_duration(Duration::from_millis(50))
                ))
            }
            SoundEffect::TetrisMove => {
                // Son subtil pour déplacement
                Some(Box::new(
                    SineWave::new(440.0) // A4
                        .take_duration(Duration::from_millis(30))
                ))
            }
            SoundEffect::TetrisHardDrop => {
                // Son de chute rapide
                Some(Box::new(
                    SquareWave::new(110.0)
                        .fade_in(Duration::from_millis(10))
                        .take_duration(Duration::from_millis(150))
                ))
            }
            SoundEffect::TetrisTetris => {
                // Son spécial pour 4 lignes (Tetris!)
                Some(Box::new(
                    SineWave::new(659.0) // E5
                        .mix(SineWave::new(784.0)) // G5
                        .mix(SineWave::new(523.0)) // C5
                        .take_duration(Duration::from_millis(600))
                ))
            }
            SoundEffect::TetrisGameOver => {
                // Son simple et triste pour game over
                Some(Box::new(
                    SquareWave::new(220.0)
                        .take_duration(Duration::from_millis(800))
                        .fade_out(Duration::from_millis(200))
                ))
            }
            
            // Pong sounds
            SoundEffect::PongPaddleHit => {
                Some(Box::new(
                    SineWave::new(600.0)
                        .take_duration(Duration::from_millis(80))
                ))
            }
            SoundEffect::PongWallHit => {
                Some(Box::new(
                    SquareWave::new(400.0)
                        .take_duration(Duration::from_millis(60))
                ))
            }
            SoundEffect::PongScore => {
                Some(Box::new(
                    SineWave::new(1200.0)
                        .take_duration(Duration::from_millis(300))
                ))
            }
            
            // Breakout sounds
            SoundEffect::BreakoutPaddleHit => {
                Some(Box::new(
                    SineWave::new(550.0)
                        .take_duration(Duration::from_millis(70))
                ))
            }
            SoundEffect::BreakoutBrickHit => {
                Some(Box::new(
                    SquareWave::new(750.0)
                        .take_duration(Duration::from_millis(120))
                ))
            }
            SoundEffect::BreakoutGameOver => {
                Some(Box::new(
                    SquareWave::new(180.0)
                        .take_duration(Duration::from_millis(600))
                ))
            }
            
            // 2048 sounds
            SoundEffect::Game2048Move => {
                Some(Box::new(
                    SineWave::new(400.0)
                        .take_duration(Duration::from_millis(100))
                ))
            }
            SoundEffect::Game2048Merge => {
                Some(Box::new(
                    SineWave::new(650.0)
                        .take_duration(Duration::from_millis(150))
                ))
            }
            SoundEffect::Game2048GameOver => {
                Some(Box::new(
                    SquareWave::new(220.0)
                        .take_duration(Duration::from_millis(700))
                ))
            }
            SoundEffect::Game2048Victory => {
                Some(Box::new(
                    SineWave::new(1400.0)
                        .take_duration(Duration::from_millis(400))
                ))
            }
            
            // Game of Life
            SoundEffect::GameOfLifeStep => {
                Some(Box::new(
                    SineWave::new(300.0)
                        .take_duration(Duration::from_millis(30))
                ))
            }
            
            // UI sounds
            SoundEffect::MenuSelect => {
                Some(Box::new(
                    SineWave::new(500.0)
                        .take_duration(Duration::from_millis(50))
                ))
            }
            SoundEffect::MenuConfirm => {
                Some(Box::new(
                    SineWave::new(800.0)
                        .take_duration(Duration::from_millis(100))
                ))
            }
        }
    }
    
    // Générer la mélodie de Tetris (Korobeiniki) - Version fidèle basée sur le tutoriel piano
    pub fn play_tetris_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }
        
        if let Some(sink) = &self.music_sink {
            let volume = *self.music_volume.lock().unwrap();
            
            // Mélodie principale (main droite) - Premier segment
            // E B C D C B A A C E D C B C D E C A A D F A G F E C E D C B B C D E C A A
            let main_melody = vec![
                (659.0, 400), // E
                (493.0, 200), // B
                (523.0, 200), // C
                (587.0, 400), // D
                (523.0, 200), // C
                (493.0, 200), // B
                (440.0, 400), // A
                (440.0, 200), // A
                (523.0, 200), // C
                (659.0, 400), // E
                (587.0, 200), // D
                (523.0, 200), // C
                (493.0, 600), // B (plus long)
                (523.0, 200), // C
                (587.0, 400), // D
                (659.0, 400), // E
                (523.0, 400), // C
                (440.0, 400), // A
                (440.0, 400), // A
                
                // Deuxième partie: D F A G F E C E D C B B C D E C A A
                (587.0, 600), // D (plus long)
                (698.0, 200), // F
                (880.0, 400), // A
                (784.0, 200), // G
                (698.0, 200), // F
                (659.0, 600), // E (plus long)
                (523.0, 200), // C
                (659.0, 400), // E
                (587.0, 200), // D
                (523.0, 200), // C
                (493.0, 400), // B
                (493.0, 200), // B
                (523.0, 200), // C
                (587.0, 400), // D
                (659.0, 400), // E
                (523.0, 400), // C
                (440.0, 400), // A
                (440.0, 400), // A
            ];
            
            // Ligne de basse simple (inspiration main gauche)
            // E E A A Ab E A D D C C E E A (simplifié)
            let bass_notes = vec![
                (329.0, 800), // E (octave plus bas)
                (220.0, 800), // A (octave plus bas)
                (207.0, 800), // Ab (octave plus bas)
                (329.0, 800), // E
                (220.0, 800), // A
                (293.0, 800), // D
                (261.0, 800), // C
                (329.0, 800), // E
            ];
            
            // Jouer la mélodie principale
            for (freq, duration_ms) in main_melody {
                let note = SineWave::new(freq)
                    .take_duration(Duration::from_millis(duration_ms))
                    .fade_in(Duration::from_millis(10))
                    .fade_out(Duration::from_millis(30))
                    .amplify(volume * 0.8); // Légèrement moins fort pour la mélodie
                sink.append(note);
            }
            
            // Ajouter quelques notes de basse en arrière-plan (plus doucement)
            for (freq, duration_ms) in bass_notes.iter().take(4) { // Seulement les 4 premières
                let bass_note = SineWave::new(*freq)
                    .take_duration(Duration::from_millis(*duration_ms))
                    .fade_in(Duration::from_millis(50))
                    .fade_out(Duration::from_millis(100))
                    .amplify(volume * 0.3); // Beaucoup plus doux pour l'accompagnement
                sink.append(bass_note);
            }
        }
    }
    
    // Méthode pour vérifier si la musique est finie et la relancer
    pub fn loop_music_if_needed(&self) {
        if self.is_music_enabled() && self.is_music_empty() {
            self.play_tetris_music();
        }
    }
    
    // Version alternative plus courte pour les niveaux rapides
    pub fn play_tetris_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }
        
        if let Some(sink) = &self.music_sink {
            let volume = *self.music_volume.lock().unwrap();
            
            // Version accélérée - notes plus courtes
            let fast_melody = vec![
                (659.0, 200), // E
                (493.0, 100), // B
                (523.0, 100), // C
                (587.0, 200), // D
                (523.0, 100), // C
                (493.0, 100), // B
                (440.0, 200), // A
                (440.0, 100), // A
                (523.0, 100), // C
                (659.0, 200), // E
                (587.0, 100), // D
                (523.0, 100), // C
                (493.0, 300), // B
                (523.0, 100), // C
                (587.0, 200), // D
                (659.0, 200), // E
                (523.0, 200), // C
                (440.0, 200), // A
                (440.0, 200), // A
            ];
            
            for (freq, duration_ms) in fast_melody {
                let note = SineWave::new(freq)
                    .take_duration(Duration::from_millis(duration_ms))
                    .fade_in(Duration::from_millis(5))
                    .fade_out(Duration::from_millis(15))
                    .amplify(volume);
                sink.append(note);
            }
        }
    }
    
    // Version avec harmonies pour les moments spéciaux (Tetris!)
    pub fn play_tetris_music_harmony(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }
        
        if let Some(sink) = &self.music_sink {
            let volume = *self.music_volume.lock().unwrap();
            
            // Version avec accord pour célébrer un Tetris
            let harmony_notes = vec![
                // Accord de victoire: E + G + C
                (659.0, 400), // E5
                (784.0, 400), // G5 (joué en même temps conceptuellement)
                (523.0, 400), // C5
                (659.0, 600), // E5 finale plus longue
            ];
            
            for (freq, duration_ms) in harmony_notes {
                let note = SineWave::new(freq)
                    .take_duration(Duration::from_millis(duration_ms))
                    .fade_in(Duration::from_millis(20))
                    .fade_out(Duration::from_millis(100))
                    .amplify(volume * 1.2); // Plus fort pour célébrer
                sink.append(note);
            }
        }
    }
    
    pub fn stop_music(&self) {
        if let Some(sink) = &self.music_sink {
            sink.clear();
        }
    }
    
    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume.clamp(0.0, 1.0);
    }
    
    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }
    
    pub fn set_music_volume(&self, volume: f32) {
        *self.music_volume.lock().unwrap() = volume.clamp(0.0, 1.0);
    }
    
    pub fn get_music_volume(&self) -> f32 {
        *self.music_volume.lock().unwrap()
    }
    
    pub fn toggle_enabled(&self) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = !*enabled;
    }
    
    pub fn set_enabled(&self, enabled: bool) {
        *self.enabled.lock().unwrap() = enabled;
    }
    
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }
    
    pub fn toggle_music(&self) {
        let mut enabled = self.music_enabled.lock().unwrap();
        *enabled = !*enabled;
        if !*enabled {
            drop(enabled);
            self.stop_music();
        }
    }
    
    pub fn set_music_enabled(&self, enabled: bool) {
        *self.music_enabled.lock().unwrap() = enabled;
        if !enabled {
            self.stop_music();
        }
    }
    
    pub fn is_music_enabled(&self) -> bool {
        *self.music_enabled.lock().unwrap()
    }
    
    pub fn clear_effects(&self) {
        if let Some(sink) = &self.effects_sink {
            sink.clear();
        }
    }
    
    // Alias pour la compatibilité
    pub fn clear_queue(&self) {
        self.clear_effects();
    }
    
    pub fn is_effects_empty(&self) -> bool {
        if let Some(sink) = &self.effects_sink {
            sink.empty()
        } else {
            true
        }
    }
    
    pub fn is_music_empty(&self) -> bool {
        if let Some(sink) = &self.music_sink {
            sink.empty()
        } else {
            true
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback silencieux si l'audio n'est pas disponible
            Self {
                _stream: None,
                effects_sink: None,
                music_sink: None,
                volume: Arc::new(Mutex::new(0.0)),
                music_volume: Arc::new(Mutex::new(0.0)),
                enabled: Arc::new(Mutex::new(false)),
                music_enabled: Arc::new(Mutex::new(false)),
            }
        })
    }
}