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

pub struct AudioManager {
    _stream: Option<OutputStream>,
    sink: Option<Sink>,
    volume: Arc<Mutex<f32>>,
    enabled: Arc<Mutex<bool>>,
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream_handle) => {
                // Dans Rodio 0.21, on connecte directement le Sink au mixer du stream
                let sink = Sink::connect_new(&stream_handle.mixer());
                
                Ok(Self {
                    _stream: Some(stream_handle),
                    sink: Some(sink),
                    volume: Arc::new(Mutex::new(0.5)),
                    enabled: Arc::new(Mutex::new(true)),
                })
            }
            Err(e) => {
                eprintln!("Erreur d'initialisation audio: {}", e);
                // Fallback en cas d'échec d'initialisation audio
                Ok(Self {
                    _stream: None,
                    sink: None,
                    volume: Arc::new(Mutex::new(0.5)),
                    enabled: Arc::new(Mutex::new(false)),
                })
            }
        }
    }
    
    pub fn play_sound(&self, effect: SoundEffect) {
        if !*self.enabled.lock().unwrap() {
            return;
        }
        
        if let Some(sink) = &self.sink {
            let volume = *self.volume.lock().unwrap();
            let source = self.generate_sound(effect);
            
            if let Some(source) = source {
                // Pour les sons importants comme Game Over, on s'assure qu'ils soient audibles
                let final_volume = match effect {
                    SoundEffect::SnakeGameOver | 
                    SoundEffect::TetrisGameOver | 
                    SoundEffect::BreakoutGameOver | 
                    SoundEffect::Game2048GameOver => volume.max(0.3), // Volume minimum pour game over
                    _ => volume,
                };
                
                sink.append(source.amplify(final_volume));
                
                // Debug: uncomment pour vérifier que la méthode est appelée
                // eprintln!("Playing sound: {:?} with volume: {}", effect, final_volume);
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
                Some(Box::new(
                    SineWave::new(1000.0)
                        .take_duration(Duration::from_millis(200))
                ))
            }
            SoundEffect::TetrisPieceDrop => {
                Some(Box::new(
                    SquareWave::new(300.0)
                        .take_duration(Duration::from_millis(50))
                ))
            }
            SoundEffect::TetrisGameOver => {
                Some(Box::new(
                    SquareWave::new(150.0)
                        .take_duration(Duration::from_millis(800))
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
    
    pub fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume.clamp(0.0, 1.0);
    }
    
    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
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
    
    // Méthode pour nettoyer le sink et éviter l'accumulation de sons
    pub fn clear_queue(&self) {
        if let Some(sink) = &self.sink {
            sink.clear();
        }
    }
    
    // Méthode pour vérifier si le sink joue encore des sons
    pub fn is_empty(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.empty()
        } else {
            true
        }
    }
    
    // Méthode de test pour vérifier que l'audio fonctionne
    pub fn test_audio(&self) {
        println!("Test audio - Enabled: {}, Volume: {}", self.is_enabled(), self.get_volume());
        if self.is_enabled() {
            self.play_sound(SoundEffect::MenuConfirm);
            println!("Son de test joué !");
        } else {
            println!("Audio désactivé !");
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback silencieux si l'audio n'est pas disponible
            Self {
                _stream: None,
                sink: None,
                volume: Arc::new(Mutex::new(0.0)),
                enabled: Arc::new(Mutex::new(false)),
            }
        })
    }
}