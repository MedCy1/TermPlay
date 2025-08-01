use crate::config::AudioConfig;
use crate::music::{
    breakout::BREAKOUT_MUSIC, gameoflife::GAMEOFLIFE_MUSIC, minesweeper::MINESWEEPER_MUSIC,
    pong::PONG_MUSIC, snake::SNAKE_MUSIC, tetris::TETRIS_MUSIC, GameMusic, _2048::GAME2048_MUSIC,
};
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

    // Minesweeper
    MinesweeperReveal,
    MinesweeperFlag,
    MinesweeperUnflag,
    MinesweeperMineHit,
    MinesweeperVictory,

    // Game of Life
    GameOfLifeStep,
    GameOfLifeCellToggle,
    GameOfLifePatternPlace,
    GameOfLifeStateChange,

    // UI
    MenuSelect,
    MenuConfirm,
    MenuBack,
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

// Gestionnaire audio global - reste en vie pendant toute l'exécution
struct GlobalAudioManager {
    _stream: OutputStream, // CRUCIAL : doit rester en vie !
    effects_sink: Sink,
    music_sink: Sink,
}

// Variable globale thread-safe avec Mutex pour macOS
static GLOBAL_AUDIO: std::sync::LazyLock<Option<GlobalAudioManager>> =
    std::sync::LazyLock::new(|| {
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream_handle) => {
                let effects_sink = Sink::connect_new(stream_handle.mixer());
                let music_sink = Sink::connect_new(stream_handle.mixer());

                Some(GlobalAudioManager {
                    _stream: stream_handle, // Garde le stream en vie !
                    effects_sink,
                    music_sink,
                })
            }
            Err(_) => None, // Fallback silencieux si pas d'audio
        }
    });

// Initialise l'audio global une seule fois
fn get_global_audio() -> Option<&'static GlobalAudioManager> {
    GLOBAL_AUDIO.as_ref()
}

pub struct AudioManager {
    master_volume: Arc<Mutex<f32>>,
    volume: Arc<Mutex<f32>>,
    music_volume: Arc<Mutex<f32>>,
    enabled: Arc<Mutex<bool>>,
    music_enabled: Arc<Mutex<bool>>,
}

impl AudioManager {
    pub fn new_with_config(config: &AudioConfig) -> Result<Self, Box<dyn std::error::Error>> {
        // Utilise le gestionnaire audio global - l'OutputStream reste en vie !
        let has_audio = get_global_audio().is_some();

        Ok(Self {
            master_volume: Arc::new(Mutex::new(config.master_volume)),
            volume: Arc::new(Mutex::new(config.effects_volume)),
            music_volume: Arc::new(Mutex::new(config.music_volume)),
            enabled: Arc::new(Mutex::new(has_audio && config.audio_enabled)),
            music_enabled: Arc::new(Mutex::new(has_audio && config.music_enabled)),
        })
    }

    pub fn play_sound(&self, effect: SoundEffect) {
        if !*self.enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let master_volume = *self.master_volume.lock().unwrap();
            let effects_volume = *self.volume.lock().unwrap();
            let source = self.generate_sound(effect);

            if let Some(source) = source {
                // Volume spécial pour certains effets
                let base_volume = match effect {
                    SoundEffect::TetrisGameOver
                    | SoundEffect::SnakeGameOver
                    | SoundEffect::BreakoutGameOver
                    | SoundEffect::Game2048GameOver => effects_volume.max(0.4),
                    SoundEffect::TetrisTetris => effects_volume * 1.2, // Plus fort pour Tetris!
                    _ => effects_volume,
                };

                // Appliquer le master volume
                let final_volume = base_volume * master_volume;
                global_audio
                    .effects_sink
                    .append(source.amplify(final_volume));
            }
        }
    }

    fn generate_sound(&self, effect: SoundEffect) -> Option<Box<dyn Source<Item = f32> + Send>> {
        match effect {
            // Snake sounds
            SoundEffect::SnakeEat => Some(Box::new(
                SineWave::new(800.0).take_duration(Duration::from_millis(100)),
            )),
            SoundEffect::SnakeGameOver => Some(Box::new(
                SquareWave::new(200.0).take_duration(Duration::from_millis(500)),
            )),

            // Tetris sounds
            SoundEffect::TetrisLineClear => {
                // Son harmonieux pour ligne complétée
                Some(Box::new(
                    SineWave::new(659.0) // E5
                        .mix(SineWave::new(523.0)) // C5
                        .take_duration(Duration::from_millis(300)),
                ))
            }
            SoundEffect::TetrisPieceDrop => {
                // Son mat pour pièce posée
                Some(Box::new(
                    SquareWave::new(220.0).take_duration(Duration::from_millis(80)),
                ))
            }
            SoundEffect::TetrisRotate => {
                // Son aigu pour rotation
                Some(Box::new(
                    SineWave::new(880.0) // A5
                        .take_duration(Duration::from_millis(50)),
                ))
            }
            SoundEffect::TetrisMove => {
                // Son subtil pour déplacement
                Some(Box::new(
                    SineWave::new(440.0) // A4
                        .take_duration(Duration::from_millis(30)),
                ))
            }
            SoundEffect::TetrisHardDrop => {
                // Son de chute rapide
                Some(Box::new(
                    SquareWave::new(110.0)
                        .fade_in(Duration::from_millis(10))
                        .take_duration(Duration::from_millis(150)),
                ))
            }
            SoundEffect::TetrisTetris => {
                // Son spécial pour 4 lignes (Tetris!)
                Some(Box::new(
                    SineWave::new(659.0) // E5
                        .mix(SineWave::new(784.0)) // G5
                        .mix(SineWave::new(523.0)) // C5
                        .take_duration(Duration::from_millis(600)),
                ))
            }
            SoundEffect::TetrisGameOver => {
                // Son simple et triste pour game over
                Some(Box::new(
                    SquareWave::new(220.0)
                        .take_duration(Duration::from_millis(800))
                        .fade_out(Duration::from_millis(200)),
                ))
            }

            // Pong sounds
            SoundEffect::PongPaddleHit => Some(Box::new(
                SineWave::new(600.0).take_duration(Duration::from_millis(80)),
            )),
            SoundEffect::PongWallHit => Some(Box::new(
                SquareWave::new(400.0).take_duration(Duration::from_millis(60)),
            )),
            SoundEffect::PongScore => Some(Box::new(
                SineWave::new(1200.0).take_duration(Duration::from_millis(300)),
            )),

            // Breakout sounds
            SoundEffect::BreakoutPaddleHit => Some(Box::new(
                SineWave::new(550.0).take_duration(Duration::from_millis(70)),
            )),
            SoundEffect::BreakoutBrickHit => Some(Box::new(
                SquareWave::new(750.0).take_duration(Duration::from_millis(120)),
            )),
            SoundEffect::BreakoutGameOver => Some(Box::new(
                SquareWave::new(180.0).take_duration(Duration::from_millis(600)),
            )),

            // 2048 sounds
            SoundEffect::Game2048Move => Some(Box::new(
                SineWave::new(400.0).take_duration(Duration::from_millis(100)),
            )),
            SoundEffect::Game2048Merge => Some(Box::new(
                SineWave::new(650.0).take_duration(Duration::from_millis(150)),
            )),
            SoundEffect::Game2048GameOver => Some(Box::new(
                SquareWave::new(220.0).take_duration(Duration::from_millis(700)),
            )),
            SoundEffect::Game2048Victory => Some(Box::new(
                SineWave::new(1400.0).take_duration(Duration::from_millis(400)),
            )),

            // Minesweeper sounds
            SoundEffect::MinesweeperReveal => {
                // Son doux pour révéler une case
                Some(Box::new(
                    SineWave::new(600.0).take_duration(Duration::from_millis(80)),
                ))
            }
            SoundEffect::MinesweeperFlag => {
                // Son de clic pour placer un drapeau
                Some(Box::new(
                    SquareWave::new(800.0).take_duration(Duration::from_millis(60)),
                ))
            }
            SoundEffect::MinesweeperUnflag => {
                // Son de clic inversé pour retirer un drapeau
                Some(Box::new(
                    SquareWave::new(600.0).take_duration(Duration::from_millis(50)),
                ))
            }
            SoundEffect::MinesweeperMineHit => {
                // Son d'explosion dramatique
                Some(Box::new(
                    SquareWave::new(150.0)
                        .mix(SquareWave::new(200.0))
                        .take_duration(Duration::from_millis(800))
                        .fade_out(Duration::from_millis(300)),
                ))
            }
            SoundEffect::MinesweeperVictory => {
                // Son de victoire triomphant
                Some(Box::new(
                    SineWave::new(1200.0).take_duration(Duration::from_millis(400)),
                ))
            }

            // Game of Life
            SoundEffect::GameOfLifeStep => Some(Box::new(
                SineWave::new(300.0).take_duration(Duration::from_millis(30)),
            )),
            SoundEffect::GameOfLifeCellToggle => {
                // Son de clic doux pour toggle de cellule
                Some(Box::new(
                    SineWave::new(440.0).take_duration(Duration::from_millis(80)),
                ))
            }
            SoundEffect::GameOfLifePatternPlace => {
                // Son harmonieux pour placement de pattern
                Some(Box::new(
                    SineWave::new(659.3) // E5
                        .mix(SineWave::new(523.3)) // C5
                        .take_duration(Duration::from_millis(150)),
                ))
            }
            SoundEffect::GameOfLifeStateChange => {
                // Son de transition pour changement d'état
                Some(Box::new(
                    SineWave::new(523.3).take_duration(Duration::from_millis(120)),
                ))
            }

            // UI sounds
            SoundEffect::MenuSelect => Some(Box::new(
                SineWave::new(500.0).take_duration(Duration::from_millis(50)),
            )),
            SoundEffect::MenuConfirm => Some(Box::new(
                SineWave::new(800.0).take_duration(Duration::from_millis(100)),
            )),
            SoundEffect::MenuBack => {
                // Son de retour - comme MenuConfirm mais descendant au lieu de montant
                Some(Box::new(
                    SineWave::new(600.0)
                        .take_duration(Duration::from_millis(80))
                        .fade_out(Duration::from_millis(30)),
                ))
            }
        }
    }

    // Jouer la musique de Tetris (version normale)
    pub fn play_tetris_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            TETRIS_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version alternative plus courte pour les niveaux rapides
    pub fn play_tetris_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            TETRIS_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version avec harmonies pour les moments spéciaux (Tetris!)
    pub fn play_tetris_music_harmony(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            TETRIS_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de Snake (version normale)
    pub fn play_snake_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            SNAKE_MUSIC.play_normal(&global_audio.music_sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            global_audio.music_sink.play();
        }
    }

    // Version rapide pour Snake (quand le serpent est très long)
    pub fn play_snake_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            SNAKE_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de Pong (version normale)
    pub fn play_pong_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            PONG_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version rapide pour Pong (quand la balle va très vite)
    pub fn play_pong_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            PONG_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Musique de célébration pour Pong
    pub fn play_pong_music_celebration(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            PONG_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de 2048 (version normale)
    pub fn play_2048_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAME2048_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version énergique pour 2048 (scores élevés/combos)
    pub fn play_2048_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAME2048_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Musique de célébration pour 2048 (victoire!)
    pub fn play_2048_music_celebration(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAME2048_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de Minesweeper (version normale)
    pub fn play_minesweeper_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            MINESWEEPER_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version tendue pour Minesweeper (moments critiques)
    pub fn play_minesweeper_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            MINESWEEPER_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Musique de célébration pour Minesweeper (victoire!)
    pub fn play_minesweeper_music_celebration(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            MINESWEEPER_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de Breakout (version normale)
    pub fn play_breakout_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            BREAKOUT_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version intense pour Breakout (peu de briques restantes)
    pub fn play_breakout_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            BREAKOUT_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Musique de célébration pour Breakout (victoire!)
    pub fn play_breakout_music_celebration(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            BREAKOUT_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Jouer la musique de Game of Life (version normale - contemplative)
    pub fn play_gameoflife_music(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAMEOFLIFE_MUSIC.play_normal(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Version dynamique pour Game of Life (simulations rapides)
    pub fn play_gameoflife_music_fast(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAMEOFLIFE_MUSIC.play_fast(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    // Musique d'émerveillement pour Game of Life (patterns complexes)
    pub fn play_gameoflife_music_celebration(&self) {
        if !*self.music_enabled.lock().unwrap() {
            return;
        }

        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.music_sink;
            let master_volume = *self.master_volume.lock().unwrap();
            let music_volume = *self.music_volume.lock().unwrap();
            let final_volume = master_volume * music_volume;
            GAMEOFLIFE_MUSIC.play_celebration(sink, final_volume);
            // Forcer le démarrage de la lecture dans Rodio 0.21
            sink.play();
        }
    }

    pub fn stop_music(&self) {
        if let Some(global_audio) = get_global_audio() {
            global_audio.music_sink.clear();
        }
    }

    pub fn set_master_volume(&self, volume: f32) {
        *self.master_volume.lock().unwrap() = volume.clamp(0.0, 1.0);
    }

    pub fn get_master_volume(&self) -> f32 {
        *self.master_volume.lock().unwrap()
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
        if let Some(global_audio) = get_global_audio() {
            let sink = &global_audio.effects_sink;
            sink.clear();
        }
    }

    pub fn is_music_empty(&self) -> bool {
        if let Some(global_audio) = get_global_audio() {
            global_audio.music_sink.empty()
        } else {
            true
        }
    }

    pub fn get_current_config(&self) -> AudioConfig {
        AudioConfig {
            master_volume: *self.master_volume.lock().unwrap(),
            effects_volume: *self.volume.lock().unwrap(),
            music_volume: *self.music_volume.lock().unwrap(),
            audio_enabled: *self.enabled.lock().unwrap(),
            music_enabled: *self.music_enabled.lock().unwrap(),
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        // Essayer de charger la configuration depuis le fichier, sinon utiliser les valeurs par défaut
        let config = crate::config::ConfigManager::new()
            .map(|config_manager| config_manager.get_audio_config().clone())
            .unwrap_or_else(|_| AudioConfig::default());

        Self::new_with_config(&config).unwrap_or_else(|_| {
            // Fallback silencieux si l'audio n'est pas disponible
            Self {
                master_volume: Arc::new(Mutex::new(config.master_volume)),
                volume: Arc::new(Mutex::new(config.effects_volume)),
                music_volume: Arc::new(Mutex::new(config.music_volume)),
                enabled: Arc::new(Mutex::new(false)), // Disable si pas d'audio hardware
                music_enabled: Arc::new(Mutex::new(false)), // Disable si pas d'audio hardware
            }
        })
    }
}

impl AudioManager {
    /// Nettoyage propre des ressources audio
    pub fn shutdown(&mut self) {
        // Arrêter juste la musique et les effets (l'OutputStream global reste en vie)
        if let Some(global_audio) = get_global_audio() {
            global_audio.effects_sink.clear();
            global_audio.music_sink.clear();
        }
        // Pas besoin de dropper l'OutputStream - il reste en vie jusqu'à la fin du programme
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}
