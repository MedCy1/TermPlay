use crate::audio::AudioManager;
use crate::config::ConfigManager;
use crate::core::{GameAction, GameInfo};
use crate::engine::fx;
use crate::highscores::HighScoreManager;
use crate::menu_ui;
use crate::music::{
    breakout::BREAKOUT_MUSIC, gameoflife::GAMEOFLIFE_MUSIC, minesweeper::MINESWEEPER_MUSIC,
    pong::PONG_MUSIC, snake::SNAKE_MUSIC, tetris::TETRIS_MUSIC, GameMusic, _2048::GAME2048_MUSIC,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    Main,
    Games,
    HighScores,
    HighScoresDetail(String), // Pour afficher les scores d'un jeu spécifique
    ConfirmClearScores(String), // Confirmation pour effacer les scores d'un jeu
    MusicPlayer,
    Settings,
    About,
}

#[derive(Debug, Clone)]
pub struct MenuOption {
    pub title: String,
    pub description: String,
    pub action: MenuAction,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    EnterSubMenu(MenuState),
    Quit,
}

pub struct MainMenu {
    current_menu: MenuState,
    menu_history: Vec<MenuState>, // Pile pour l'historique de navigation
    main_options: Vec<MenuOption>,
    games_list: Vec<GameInfo>,
    selected_index: usize,
    list_state: ListState,
    audio: AudioManager,
    config_manager: ConfigManager,
    highscore_manager: HighScoreManager,
    music_tracks: Vec<MusicTrack>,
    current_playing: Option<usize>,
    current_variant: Vec<usize>, // Index de la variante sélectionnée pour chaque track

    // Animation
    time: f32,
    sel_pos: f32, // position (fractionnaire) de la barre de sélection
    prev_menu: MenuState,
    prev_sel: usize,
    card_t: f32,  // 0 → 1 après un changement de sélection (éclaire la carte)
    trans: f32,   // 1 → 0 pendant le glissement entre écrans (entrées ignorées)
    last: Buffer, // dernier écran affiché (snapshot de l'écran sortant)
    slide_old: Option<Buffer>,
    settings_tab: usize,
    slide_dir: i32, // +1: l'ancien écran part à gauche (avancer), -1: à droite (retour)
}

/// Durée du glissement entre deux écrans du menu (lisible, mais vif au clavier).
const SCREEN_FADE: f32 = 0.13;

#[derive(Debug, Clone)]
pub struct MusicTrack {
    pub name: String,
    pub variants: Vec<String>, // normal, fast, celebration
}

impl MainMenu {
    pub fn new(games: Vec<&GameInfo>) -> Result<Self, Box<dyn std::error::Error>> {
        // Charger la configuration
        let config_manager = ConfigManager::new()?;
        let audio_config = config_manager.get_audio_config();
        let main_options = vec![
            MenuOption {
                title: "🎮 Games".to_string(),
                description: "Play exciting terminal games".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::Games),
            },
            MenuOption {
                title: "🏆 High Scores".to_string(),
                description: "View best scores and leaderboards".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::HighScores),
            },
            MenuOption {
                title: "🎵 Music Player".to_string(),
                description: "Listen to game soundtracks".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::MusicPlayer),
            },
            MenuOption {
                title: "⚙️ Settings".to_string(),
                description: "Configure game preferences".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::Settings),
            },
            MenuOption {
                title: "ℹ️ About".to_string(),
                description: "About TermPlay".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::About),
            },
            MenuOption {
                title: "🚪 Quit".to_string(),
                description: "Exit TermPlay".to_string(),
                action: MenuAction::Quit,
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let music_tracks = vec![
            MusicTrack {
                name: TETRIS_MUSIC.name().to_string(),
                variants: vec![
                    "Normal".to_string(),
                    "Fast".to_string(),
                    "Celebration".to_string(),
                ],
            },
            MusicTrack {
                name: SNAKE_MUSIC.name().to_string(),
                variants: vec!["Normal".to_string(), "Fast".to_string()],
            },
            MusicTrack {
                name: PONG_MUSIC.name().to_string(),
                variants: vec![
                    "Normal".to_string(),
                    "Fast".to_string(),
                    "Celebration".to_string(),
                ],
            },
            MusicTrack {
                name: GAME2048_MUSIC.name().to_string(),
                variants: vec![
                    "Normal".to_string(),
                    "Fast".to_string(),
                    "Celebration".to_string(),
                ],
            },
            MusicTrack {
                name: MINESWEEPER_MUSIC.name().to_string(),
                variants: vec![
                    "Normal".to_string(),
                    "Intense".to_string(),
                    "Victory".to_string(),
                ],
            },
            MusicTrack {
                name: BREAKOUT_MUSIC.name().to_string(),
                variants: vec![
                    "Normal".to_string(),
                    "Intense".to_string(),
                    "Victory".to_string(),
                ],
            },
            MusicTrack {
                name: GAMEOFLIFE_MUSIC.name().to_string(),
                variants: vec![
                    "Contemplative".to_string(),
                    "Dynamic".to_string(),
                    "Wonder".to_string(),
                ],
            },
        ];

        // Créer l'AudioManager avec la configuration chargée
        let audio = AudioManager::new_with_config(audio_config)?;

        // Créer le HighScoreManager
        let highscore_manager = HighScoreManager::new().unwrap_or_default();

        // Initialiser les variantes sélectionnées (index 0 = première variante pour chaque track)
        let current_variant = vec![0; music_tracks.len()];

        Ok(Self {
            current_menu: MenuState::Main,
            menu_history: Vec::new(), // Initialiser la pile vide
            main_options,
            games_list: games.into_iter().cloned().collect(),
            selected_index: 0,
            list_state,
            audio,
            config_manager,
            highscore_manager,
            music_tracks,
            current_playing: None,
            current_variant,
            time: 0.0,
            sel_pos: 0.0,
            prev_menu: MenuState::Main,
            prev_sel: 0,
            card_t: 1.0,
            trans: 0.0,
            last: Buffer::default(),
            slide_old: None,
            slide_dir: 1,
            settings_tab: 0,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        // Pendant le fondu entre écrans, les touches sont ignorées (évite les doubles appuis)
        if fx::fx_enabled() && self.trans > 0.0 {
            return GameAction::Continue;
        }
        match key.code {
            KeyCode::Char('q') => {
                if self.current_menu == MenuState::Main {
                    self.audio
                        .play_sound(crate::audio::SoundEffect::MenuConfirm);
                    GameAction::Quit
                } else {
                    // Arrêter la musique si on quitte le music player
                    if self.current_menu == MenuState::MusicPlayer {
                        self.audio.stop_music();
                        self.current_playing = None;
                    }
                    self.audio.play_sound(crate::audio::SoundEffect::MenuBack);
                    self.go_back();
                    GameAction::Continue
                }
            }
            KeyCode::Esc => {
                if self.current_menu != MenuState::Main {
                    self.audio.play_sound(crate::audio::SoundEffect::MenuBack);
                    self.go_back();
                }
                GameAction::Continue
            }
            KeyCode::Tab | KeyCode::BackTab if self.current_menu == MenuState::Settings => {
                let dir = if key.code == KeyCode::Tab { 1 } else { -1 };
                self.switch_settings_tab(dir);
                GameAction::Continue
            }
            KeyCode::Char(c @ '1'..='3') if self.current_menu == MenuState::Settings => {
                let t = c as usize - '1' as usize;
                self.switch_settings_tab(t as i32 - self.settings_tab as i32);
                GameAction::Continue
            }
            KeyCode::Down => {
                self.next_item();
                self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                GameAction::Continue
            }
            KeyCode::Up => {
                self.previous_item();
                self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                GameAction::Continue
            }
            KeyCode::Left => {
                if self.current_menu == MenuState::MusicPlayer {
                    self.previous_variant();
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                } else if self.current_menu == MenuState::Settings {
                    self.adjust_setting(-1, false);
                }
                GameAction::Continue
            }
            KeyCode::Right => {
                if self.current_menu == MenuState::MusicPlayer {
                    self.next_variant();
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                } else if self.current_menu == MenuState::Settings {
                    self.adjust_setting(1, false);
                }
                GameAction::Continue
            }
            KeyCode::Enter => {
                self.audio
                    .play_sound(crate::audio::SoundEffect::MenuConfirm);
                self.select_current_item()
            }
            KeyCode::Char(' ') => {
                if self.current_menu == MenuState::MusicPlayer {
                    self.audio
                        .play_sound(crate::audio::SoundEffect::MenuConfirm);
                    self.play_selected_music();
                }
                GameAction::Continue
            }
            KeyCode::Char('s') => {
                if self.current_menu == MenuState::MusicPlayer {
                    self.audio.stop_music();
                    self.current_playing = None;
                }
                GameAction::Continue
            }
            KeyCode::Char('c') => {
                // Clear scores - demander confirmation
                if let MenuState::HighScoresDetail(game_name) = &self.current_menu {
                    self.navigate_to(MenuState::ConfirmClearScores(game_name.clone()));
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                }
                GameAction::Continue
            }
            KeyCode::Char('y') => {
                // Confirmer la suppression
                if let MenuState::ConfirmClearScores(game_name) = &self.current_menu {
                    if let Err(e) = self.highscore_manager.clear_game_scores(game_name) {
                        eprintln!("Error clearing scores: {e}");
                    }
                    // Recharger les scores depuis le disque pour rafraîchir l'affichage
                    if let Err(e) = self.highscore_manager.reload() {
                        eprintln!("Error reloading scores: {e}");
                    }
                    self.audio
                        .play_sound(crate::audio::SoundEffect::MenuConfirm);
                    // Retourner à la liste des high scores
                    self.go_back(); // Retour au HighScoresDetail
                    self.go_back(); // Retour au HighScores
                }
                GameAction::Continue
            }
            KeyCode::Char('n') => {
                // Annuler la suppression
                if let MenuState::ConfirmClearScores(_) = &self.current_menu {
                    self.audio.play_sound(crate::audio::SoundEffect::MenuBack);
                    self.go_back();
                }
                GameAction::Continue
            }
            _ => GameAction::Continue,
        }
    }

    fn next_item(&mut self) {
        let max_items = match &self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::HighScores => {
                let games_with_scores = self.highscore_manager.get_games_with_scores();
                games_with_scores.len().max(1) // Au moins 1 pour "No scores yet"
            }
            MenuState::HighScoresDetail(game_name) => {
                // Récupérer le nombre réel de scores pour ce jeu
                let scores = self.highscore_manager.get_scores(game_name);
                scores.len().max(1) // Au moins 1 pour "No scores yet"
            }
            MenuState::ConfirmClearScores(_) => 2, // Yes/No
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => crate::settings::row_count(self.settings_tab),
            MenuState::About => 1,
        };

        if max_items == 0 {
            return;
        }

        self.selected_index = (self.selected_index + 1) % max_items;
        self.list_state.select(Some(self.selected_index));
    }

    fn previous_item(&mut self) {
        let max_items = match &self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::HighScores => {
                let games_with_scores = self.highscore_manager.get_games_with_scores();
                games_with_scores.len().max(1) // Au moins 1 pour "No scores yet"
            }
            MenuState::HighScoresDetail(game_name) => {
                // Récupérer le nombre réel de scores pour ce jeu
                let scores = self.highscore_manager.get_scores(game_name);
                scores.len().max(1) // Au moins 1 pour "No scores yet"
            }
            MenuState::ConfirmClearScores(_) => 2, // Yes/No
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => crate::settings::row_count(self.settings_tab),
            MenuState::About => 1,
        };

        if max_items == 0 {
            return;
        }

        self.selected_index = if self.selected_index == 0 {
            max_items - 1
        } else {
            self.selected_index - 1
        };
        self.list_state.select(Some(self.selected_index));
    }

    fn select_current_item(&mut self) -> GameAction {
        match self.current_menu {
            MenuState::Main => {
                if let Some(option) = self.main_options.get(self.selected_index) {
                    match &option.action {
                        MenuAction::EnterSubMenu(menu_state) => {
                            self.navigate_to(menu_state.clone());
                            GameAction::Continue
                        }
                        MenuAction::Quit => GameAction::Quit,
                    }
                } else {
                    GameAction::Continue
                }
            }
            MenuState::Games => {
                if let Some(_game) = self.games_list.get(self.selected_index) {
                    GameAction::GameOver
                } else {
                    GameAction::Continue
                }
            }
            MenuState::MusicPlayer => {
                self.play_selected_music();
                GameAction::Continue
            }
            MenuState::Settings => {
                // Entrée: bascule les interrupteurs, fait défiler les choix
                self.adjust_setting(1, true);
                GameAction::Continue
            }
            MenuState::HighScores => {
                let games_with_scores = self.highscore_manager.get_games_with_scores();
                if let Some(game_name) = games_with_scores.get(self.selected_index) {
                    self.navigate_to(MenuState::HighScoresDetail(game_name.clone()));
                }
                GameAction::Continue
            }
            MenuState::HighScoresDetail(_) => {
                // Retour à la liste des high scores
                self.go_back();
                GameAction::Continue
            }
            MenuState::ConfirmClearScores(_) => {
                // Enter ne fait rien ici, utiliser Y/N
                GameAction::Continue
            }
            MenuState::About => {
                self.go_back();
                GameAction::Continue
            }
        }
    }

    /// Navigue vers un nouveau menu en sauvegardant l'état actuel dans la pile
    /// Démarre un glissement depuis le dernier écran affiché (sans effet avec `--no-fx`).
    fn start_slide(&mut self, dir: i32) {
        if fx::fx_enabled() && !self.last.content.is_empty() {
            self.slide_old = Some(self.last.clone());
            self.slide_dir = dir;
            self.trans = 1.0;
        }
    }

    fn navigate_to(&mut self, new_menu: MenuState) {
        // Recharger les scores si on entre dans le menu High Scores
        if matches!(
            new_menu,
            MenuState::Games | MenuState::HighScores | MenuState::HighScoresDetail(_)
        ) {
            if let Err(e) = self.highscore_manager.reload() {
                eprintln!("Error reloading scores: {e}");
            }
        }

        self.start_slide(1);
        if new_menu == MenuState::Settings {
            self.settings_tab = 0;
        }

        // Sauvegarder le menu actuel dans la pile
        self.menu_history.push(self.current_menu.clone());
        // Passer au nouveau menu
        self.current_menu = new_menu;
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    fn go_back(&mut self) {
        self.start_slide(-1);
        // Remonter d'un niveau en utilisant la pile
        if let Some(previous_menu) = self.menu_history.pop() {
            self.current_menu = previous_menu;
        } else {
            // Si la pile est vide, retourner au menu principal
            self.current_menu = MenuState::Main;
        }
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    fn next_variant(&mut self) {
        if let Some(track) = self.music_tracks.get(self.selected_index) {
            if !track.variants.is_empty() {
                let current = &mut self.current_variant[self.selected_index];
                *current = (*current + 1) % track.variants.len();
            }
        }
    }

    fn previous_variant(&mut self) {
        if let Some(track) = self.music_tracks.get(self.selected_index) {
            if !track.variants.is_empty() {
                let current = &mut self.current_variant[self.selected_index];
                *current = if *current == 0 {
                    track.variants.len() - 1
                } else {
                    *current - 1
                };
            }
        }
    }

    fn switch_settings_tab(&mut self, delta: i32) {
        let n = crate::settings::TABS.len() as i32;
        let next = (self.settings_tab as i32 + delta).rem_euclid(n) as usize;
        if next == self.settings_tab {
            return;
        }
        self.start_slide(if delta > 0 { 1 } else { -1 });
        self.settings_tab = next;
        self.selected_index = 0;
        self.list_state.select(Some(0));
        self.sel_pos = 0.0;
        self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
    }

    /// Valeurs audio courantes (master, effets, musique, effets actifs, musique active).
    fn audio_values(&self) -> (f32, f32, f32, bool, bool) {
        (
            self.audio.get_master_volume(),
            self.audio.get_volume(),
            self.audio.get_music_volume(),
            self.audio.is_enabled(),
            self.audio.is_music_enabled(),
        )
    }

    /// Modifie le réglage sélectionné: `dir` = ±1; `enter` = touche Entrée (ne touche pas aux curseurs).
    fn adjust_setting(&mut self, dir: i32, enter: bool) {
        use crate::config::{ColorPref, FxMode};
        let step = |v: f32| ((v * 10.0).round() + dir as f32).clamp(0.0, 10.0) / 10.0;
        let cycle = |i: usize, n: usize| (i as i32 + dir).rem_euclid(n as i32) as usize;
        let mut audio_changed = false;
        let mut cfg_changed = false;
        let mut feedback = true;
        let row = self.selected_index;
        match (self.settings_tab, row) {
            (0, 0) => {
                let i = cycle(crate::settings::fx_index(self.config_manager.get()), 3);
                self.update_config(|c| {
                    c.visuals.fx_mode = [FxMode::Full, FxMode::Low, FxMode::Disabled][i]
                });
                cfg_changed = true;
            }
            (0, 1) => {
                let i = cycle(crate::settings::color_index(self.config_manager.get()), 4);
                self.update_config(|c| {
                    c.visuals.color_mode = [
                        ColorPref::Auto,
                        ColorPref::TrueColor,
                        ColorPref::Palette256,
                        ColorPref::Mono,
                    ][i]
                });
                cfg_changed = true;
            }
            (0, 2) => {
                let i = cycle(crate::settings::shake_index(self.config_manager.get()), 3);
                self.update_config(|c| c.visuals.shake_percent = [0, 50, 100][i]);
                cfg_changed = true;
            }
            (1, 0) if !enter => {
                self.audio
                    .set_master_volume(step(self.audio.get_master_volume()));
                audio_changed = true;
            }
            (1, 1) if !enter => {
                self.audio.set_volume(step(self.audio.get_volume()));
                audio_changed = true;
            }
            (1, 2) if !enter => {
                self.audio
                    .set_music_volume(step(self.audio.get_music_volume()));
                audio_changed = true;
            }
            (1, 3) => {
                // ←/→ allument et éteignent, Entrée bascule
                let on = if enter {
                    !self.audio.is_enabled()
                } else {
                    dir > 0
                };
                self.audio.set_enabled(on);
                audio_changed = true;
            }
            (1, 4) => {
                let on = if enter {
                    !self.audio.is_music_enabled()
                } else {
                    dir > 0
                };
                self.audio.set_music_enabled(on);
                audio_changed = true;
            }
            (2, 0) => {
                let on = if enter {
                    !self.config_manager.get().gameplay.ghost_piece
                } else {
                    dir > 0
                };
                self.update_config(|c| c.gameplay.ghost_piece = on);
                cfg_changed = true;
            }
            (2, 1) => {
                let on = if enter {
                    !self.config_manager.get().gameplay.floating_scores
                } else {
                    dir > 0
                };
                self.update_config(|c| c.gameplay.floating_scores = on);
                cfg_changed = true;
            }
            _ => feedback = false,
        }
        if audio_changed {
            self.save_audio_config();
        }
        // Micro-feedback sonore: le bip suit le nouveau volume des effets
        if feedback && (audio_changed || cfg_changed) {
            self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
        }
    }

    fn update_config(&mut self, f: impl FnOnce(&mut crate::config::GameConfig)) {
        if let Err(e) = self.config_manager.update(f) {
            eprintln!("Erreur lors de la sauvegarde de la configuration: {e}");
        }
    }

    fn save_audio_config(&mut self) {
        let current_audio_config = self.audio.get_current_config();
        if let Err(e) = self.config_manager.update_audio_config(|config| {
            *config = current_audio_config;
        }) {
            eprintln!("Erreur lors de la sauvegarde de la configuration audio: {e}");
        }
    }

    /// Jouer une musique à un index spécifique
    fn play_music_at_index(&mut self, track_index: usize) {
        if let Some(track) = self.music_tracks.get(track_index) {
            self.audio.stop_music(); // Arrêter toute musique en cours

            // S'assurer que l'audio est activé
            if !self.audio.is_enabled() {
                self.audio.set_enabled(true);
            }
            if !self.audio.is_music_enabled() {
                self.audio.set_music_enabled(true);
            }

            // Jouer la musique sélectionnée avec la variante choisie
            let variant_index = self.current_variant[track_index];

            match track.name.as_str() {
                "Tetris (Korobeiniki)" => {
                    match variant_index {
                        0 => self.audio.play_tetris_music(),         // Normal
                        1 => self.audio.play_tetris_music_fast(),    // Fast
                        2 => self.audio.play_tetris_music_harmony(), // Celebration
                        _ => self.audio.play_tetris_music(),
                    }
                }
                "Snake Ambient" => {
                    match variant_index {
                        0 => self.audio.play_snake_music(),      // Normal
                        1 => self.audio.play_snake_music_fast(), // Fast
                        _ => self.audio.play_snake_music(),
                    }
                }
                "Pong Retro Electronic" => {
                    match variant_index {
                        0 => self.audio.play_pong_music(),             // Normal
                        1 => self.audio.play_pong_music_fast(),        // Fast
                        2 => self.audio.play_pong_music_celebration(), // Celebration
                        _ => self.audio.play_pong_music(),
                    }
                }
                "2048 Zen Mode" => {
                    match variant_index {
                        0 => self.audio.play_2048_music(),             // Normal
                        1 => self.audio.play_2048_music_fast(),        // Fast
                        2 => self.audio.play_2048_music_celebration(), // Celebration
                        _ => self.audio.play_2048_music(),
                    }
                }
                "Minesweeper Tension" => {
                    match variant_index {
                        0 => self.audio.play_minesweeper_music(),      // Normal
                        1 => self.audio.play_minesweeper_music_fast(), // Intense
                        2 => self.audio.play_minesweeper_music_celebration(), // Victory
                        _ => self.audio.play_minesweeper_music(),
                    }
                }
                "Breakout Arcade" => {
                    match variant_index {
                        0 => self.audio.play_breakout_music(),             // Normal
                        1 => self.audio.play_breakout_music_fast(),        // Intense
                        2 => self.audio.play_breakout_music_celebration(), // Victory
                        _ => self.audio.play_breakout_music(),
                    }
                }
                "Game of Life Ambient" => {
                    match variant_index {
                        0 => self.audio.play_gameoflife_music(), // Contemplative
                        1 => self.audio.play_gameoflife_music_fast(), // Dynamic
                        2 => self.audio.play_gameoflife_music_celebration(), // Wonder
                        _ => self.audio.play_gameoflife_music(),
                    }
                }
                _ => {}
            }

            self.current_playing = Some(track_index);
        }
    }

    /// Jouer la musique actuellement sélectionnée
    fn play_selected_music(&mut self) {
        self.play_music_at_index(self.selected_index);
    }

    /// Rejouer la musique qui est actuellement en cours de lecture
    fn replay_current_music(&mut self) {
        if let Some(playing_index) = self.current_playing {
            self.play_music_at_index(playing_index);
        }
    }

    pub fn get_selected_game(&self) -> Option<&str> {
        if self.current_menu == MenuState::Games {
            self.games_list
                .get(self.selected_index)
                .map(|g| g.name.as_str())
        } else {
            None
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        draw_main_menu(frame, self);
    }

    /// Avance les animations (60 fps): temps, glissement de la sélection, éclat de la carte.
    pub fn animate(&mut self, dt: std::time::Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.time += dt;
        self.trans = if fx::fx_enabled() {
            (self.trans - dt / SCREEN_FADE).max(0.0)
        } else {
            0.0
        };
        let target = self.selected_index as f32;
        if !fx::fx_enabled() || self.prev_menu != self.current_menu {
            self.sel_pos = target;
            self.prev_menu = self.current_menu.clone();
            self.prev_sel = self.selected_index;
            self.card_t = 1.0;
            return;
        }
        self.sel_pos += (target - self.sel_pos) * (1.0 - (-18.0 * dt).exp());
        if self.prev_sel != self.selected_index {
            self.prev_sel = self.selected_index;
            self.card_t = 0.0;
        }
        self.card_t = (self.card_t + dt / 0.3).min(1.0);
    }

    pub fn update(&mut self) {
        // Gérer la boucle de musique si on est dans le music player
        if self.current_menu == MenuState::MusicPlayer
            && self.current_playing.is_some()
            && self.audio.is_music_enabled()
            && self.audio.is_music_empty()
        {
            // Relancer la musique qui était en cours de lecture (pas celle sélectionnée)
            self.replay_current_music();
        }
    }

    /// Nettoie les ressources audio avant fermeture
    pub fn cleanup_audio(&mut self) {
        self.audio.shutdown();
    }
}

fn draw_main_menu(frame: &mut Frame, app: &mut MainMenu) {
    let area = frame.area();

    // Fond sombre élégant
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // Layout simple et propre
    let banner = app.current_menu == MenuState::Main && area.height >= 24;
    let chunks = Layout::vertical([
        Constraint::Length(if banner { 8 } else { 4 }), // Header
        Constraint::Min(0),                             // Zone principale
        Constraint::Length(3),                          // Footer
    ])
    .split(area);

    // === HEADER ===
    let title = match &app.current_menu {
        MenuState::Main => "TERMPLAY",
        MenuState::Games => "GAMES",
        MenuState::HighScores => "HIGH SCORES",
        MenuState::HighScoresDetail(_) => "LEADERBOARD",
        MenuState::ConfirmClearScores(_) => "CONFIRM DELETION",
        MenuState::MusicPlayer => "MUSIC PLAYER",
        MenuState::Settings => "SETTINGS",
        MenuState::About => "ABOUT",
    };

    let subtitle = match &app.current_menu {
        MenuState::Main => "Terminal Mini-Games Collection".to_string(),
        MenuState::Games => "Choose your adventure".to_string(),
        MenuState::HighScores => "Best scores and achievements".to_string(),
        MenuState::HighScoresDetail(game_name) => format!("Top scores for {game_name}"),
        MenuState::ConfirmClearScores(game_name) => {
            format!("Are you sure you want to delete all scores for {game_name}?")
        }
        MenuState::MusicPlayer => "Listen to game soundtracks".to_string(),
        MenuState::Settings => "Visuals, audio and gameplay".to_string(),
        MenuState::About => "Information about TermPlay".to_string(),
    };

    let header_text = vec![
        Line::from(vec![
            "🎮 ".cyan().bold(),
            title.yellow().bold(),
            " 🎮".cyan().bold(),
        ]),
        Line::from(subtitle.as_str().magenta()),
    ];

    if banner {
        let head = chunks[0];
        menu_ui::draw_banner(
            frame.buffer_mut(),
            Rect::new(head.x, head.y + 1, head.width, head.height - 1),
            app.time,
        );
        let sub =
            Paragraph::new(Line::from(subtitle.as_str().magenta())).alignment(Alignment::Center);
        frame.render_widget(sub, Rect::new(head.x, head.y + 6, head.width, 1));
    } else {
        let header = Paragraph::new(header_text)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Game Status ".white().bold())
                    .border_style(Style::new().cyan())
                    .style(Style::default().bg(Color::Rgb(25, 35, 45))),
            );
        frame.render_widget(header, chunks[0]);
    }

    // === ZONE PRINCIPALE ===
    match &app.current_menu {
        MenuState::Main => draw_main_options(frame, chunks[1], app),
        MenuState::Games => draw_games_menu(frame, chunks[1], app),
        MenuState::HighScores => draw_highscores_menu(frame, chunks[1], app),
        MenuState::HighScoresDetail(game_name) => {
            let game_name_clone = game_name.clone();
            draw_highscores_detail(frame, chunks[1], app, &game_name_clone)
        }
        MenuState::ConfirmClearScores(game_name) => {
            let game_name_clone = game_name.clone();
            draw_confirm_clear_scores(frame, chunks[1], &game_name_clone)
        }
        MenuState::MusicPlayer => draw_music_player(frame, chunks[1], app),
        MenuState::Settings => {
            let rows = crate::settings::rows(
                app.settings_tab,
                app.config_manager.get(),
                app.audio_values(),
            );
            crate::settings::draw(
                frame,
                chunks[1],
                app.settings_tab,
                &rows,
                app.selected_index,
                app.sel_pos,
                app.time,
            );
        }
        MenuState::About => draw_about_menu(frame, chunks[1]),
    }

    // === FOOTER ===
    let controls = match app.current_menu {
        MenuState::Main => "Arrow Keys Move • Enter Select • Q Quit",
        MenuState::MusicPlayer => {
            "↑↓ Select Track • ←→ Change Variant • Space/Enter Play • S Stop • Esc/Q Back"
        }
        MenuState::Settings => {
            "↑↓ Select • ←→ Adjust • Enter Toggle • Tab/1-3 Switch tab • Esc/Q Back"
        }
        MenuState::HighScoresDetail(_) => "C Clear Scores • Esc/Q Back",
        MenuState::ConfirmClearScores(_) => "Y Yes • N No",
        _ => "Arrow Keys Move • Enter Select • Esc/Q Back",
    };

    let footer_text = vec![Line::from(vec![
        "Controls: ".gray(),
        controls.white().bold(),
    ])];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(footer, chunks[2]);

    // Poussières d'ambiance, par-dessus tout mais seulement sur les cellules vides
    menu_ui::draw_motes(frame.buffer_mut(), area, app.time);

    // Glissement entre écrans: l'ancien sort, le nouveau entre (ease-out), sans passer par le noir
    let buf = frame.buffer_mut();
    match (&app.slide_old, app.trans > 0.0) {
        (Some(old), true) if old.area == buf.area => {
            let w = buf.area.width as usize;
            let p = 1.0 - app.trans;
            let off = (((1.0 - (1.0 - p) * (1.0 - p)) * w as f32).round() as usize).min(w);
            let new = buf.content.clone();
            for (i, cell) in buf.content.iter_mut().enumerate() {
                let x = i % w;
                let src = if app.slide_dir > 0 {
                    if x + off < w {
                        Some(&old.content[i + off])
                    } else {
                        Some(&new[i + off - w])
                    }
                } else if x >= off {
                    Some(&old.content[i - off])
                } else {
                    Some(&new[i + w - off])
                };
                if let Some(c) = src {
                    *cell = c.clone();
                }
            }
        }
        (_, false) => {
            app.slide_old = None;
            app.last.clone_from(buf); // réutilise l'allocation
        }
        _ => {}
    }
}

fn draw_main_options(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let rows: Vec<Line<'static>> = app
        .main_options
        .iter()
        .map(|o| {
            Line::from(vec![
                Span::styled(o.title.clone(), Style::default().fg(Color::White).bold()),
                Span::styled("  -  ", Style::default().fg(Color::Gray)),
                Span::styled(o.description.clone(), Style::default().fg(Color::LightBlue)),
            ])
        })
        .collect();
    menu_ui::draw_selector(
        frame,
        area,
        " Main Menu ",
        Color::Green,
        (60, 140, 255),
        &rows,
        app.selected_index,
        app.sel_pos,
        app.time,
    );
}

fn draw_games_menu(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let with_card = area.width >= 76;
    let (list_area, card_area) = if with_card {
        let c = Layout::horizontal([Constraint::Min(30), Constraint::Length(38)]).split(area);
        (c[0], Some(c[1]))
    } else {
        (area, None)
    };

    let rows: Vec<Line<'static>> = app
        .games_list
        .iter()
        .map(|game| {
            let (icon, accent, _) = menu_ui::game_meta(&crate::games::key(&game.name));
            let mut spans = vec![
                Span::styled(
                    format!("{icon} "),
                    Style::default().fg(fx::color(accent)).bold(),
                ),
                Span::styled(
                    game.name.to_uppercase(),
                    Style::default().fg(Color::White).bold(),
                ),
            ];
            if !with_card {
                spans.push(Span::styled("  -  ", Style::default().fg(Color::Gray)));
                spans.push(Span::styled(
                    game.description.clone(),
                    Style::default().fg(Color::LightBlue),
                ));
            }
            Line::from(spans)
        })
        .collect();
    let selected = app.games_list.get(app.selected_index).cloned();
    let accent = selected.as_ref().map_or((50, 210, 110), |g| {
        menu_ui::game_meta(&crate::games::key(&g.name)).1
    });
    menu_ui::draw_selector(
        frame,
        list_area,
        " Available Games ",
        Color::Green,
        accent,
        &rows,
        app.selected_index,
        app.sel_pos,
        app.time,
    );

    if let (Some(card), Some(game)) = (card_area, selected) {
        let key = crate::games::key(&game.name);
        let best = app.highscore_manager.get_best_score(&key).map(|s| s.score);
        menu_ui::draw_card(
            frame,
            card,
            &key,
            &game.name,
            &game.description,
            best,
            app.card_t,
            app.time,
        );
    }
}

fn draw_about_menu(frame: &mut Frame, area: Rect) {
    // Récupérer la version depuis Cargo.toml automatiquement
    let version = env!("CARGO_PKG_VERSION");
    let version_text = format!("🎮 TermPlay v{version}");

    let about_text = vec![
        Line::from(""),
        Line::from(version_text.cyan().bold()),
        Line::from(""),
        Line::from("A beautiful collection of terminal mini-games"),
        Line::from("built with Rust and Ratatui."),
        Line::from(""),
        Line::from("Features:".yellow().bold()),
        Line::from("• Classic games with modern graphics"),
        Line::from("• Responsive design that adapts to terminal size"),
        Line::from("• Extensible architecture for adding new games"),
        Line::from(""),
        Line::from("Created with ❤️ by MedCy1 using Rust".red()),
    ];

    let about = Paragraph::new(about_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" About TermPlay ".cyan().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        );
    frame.render_widget(about, area);
}

fn draw_music_player(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let items: Vec<ListItem> = app
        .music_tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let status = if app.current_playing == Some(i) {
                "▶️ "
            } else {
                "🎵 "
            };

            let playing_text = if app.current_playing == Some(i) {
                " [PLAYING]".green().bold()
            } else {
                "".white()
            };

            // Afficher la variante actuellement sélectionnée en surbrillance
            let current_variant_idx = app.current_variant[i];
            let mut variants_display = Vec::new();
            for (idx, variant) in track.variants.iter().enumerate() {
                if idx == current_variant_idx {
                    variants_display.push(format!("[{variant}]")); // Variante sélectionnée
                } else {
                    variants_display.push(variant.clone());
                }
            }
            let variants_text = format!(" ({})", variants_display.join(", "));

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("  {status} "),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::styled(&track.name, Style::default().fg(Color::White).bold()),
                Span::styled(variants_text, Style::default().fg(Color::Gray)),
                playing_text,
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Available Music Tracks ".magenta().bold())
                .border_style(Style::new().magenta())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(100, 0, 150))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_highscores_menu(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let games_with_scores = app.highscore_manager.get_games_with_scores();

    if games_with_scores.is_empty() {
        // Aucun score enregistré
        let paragraph =
            Paragraph::new("🏆 No high scores yet!\n\nPlay some games to see your scores here.")
                .block(
                    Block::bordered()
                        .title(" High Scores ".yellow().bold())
                        .border_style(Style::new().yellow())
                        .style(Style::default().bg(Color::Rgb(10, 15, 20))),
                )
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = games_with_scores
        .iter()
        .map(|game_name| {
            let best_score = app.highscore_manager.get_best_score(game_name);
            let score_text = if let Some(score) = best_score {
                format!(" (Best: {})", score.score)
            } else {
                " (No scores)".to_string()
            };

            let content = vec![Line::from(vec![
                Span::styled("  🎮 ", Style::default().fg(Color::Yellow)),
                Span::styled(game_name, Style::default().fg(Color::White).bold()),
                Span::styled(score_text, Style::default().fg(Color::Gray)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Games with High Scores ".yellow().bold())
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(200, 200, 0))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_highscores_detail(frame: &mut Frame, area: Rect, app: &mut MainMenu, game_name: &str) {
    let scores = app.highscore_manager.get_scores(game_name);

    if scores.is_empty() {
        let paragraph = Paragraph::new(format!(
            "🏆 No scores yet for {game_name}!\n\nPlay this game to set your first high score."
        ))
        .block(
            Block::bordered()
                .title(format!(" {game_name} Leaderboard ").yellow().bold())
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = scores
        .iter()
        .enumerate()
        .map(|(index, score)| {
            let rank = index + 1;
            let medal = match rank {
                1 => "🥇",
                2 => "🥈",
                3 => "🥉",
                _ => "🏅",
            };

            let player_name = if score.player_name.is_empty() {
                "Anonymous"
            } else {
                &score.player_name
            };

            let content = vec![Line::from(vec![
                Span::styled(format!(" {medal}  "), Style::default()),
                Span::styled(
                    format!("#{rank:<2} "),
                    Style::default().fg(Color::Yellow).bold(),
                ),
                Span::styled(
                    format!("{player_name:<15} "),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::styled(
                    format!("{:>8} pts", score.score),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::styled(
                    format!("  {}", score.format_duration()),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(
                    format!("  {}", score.format_date()),
                    Style::default().fg(Color::DarkGray),
                ),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(
                    format!(" {} - Top {} ", game_name, scores.len())
                        .yellow()
                        .bold(),
                )
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(200, 200, 0))
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_confirm_clear_scores(frame: &mut Frame, area: Rect, game_name: &str) {
    let confirmation_text = vec![
        Line::from(""),
        Line::from(""),
        Line::from("⚠️  WARNING  ⚠️".red().bold()),
        Line::from(""),
        Line::from(vec![
            "You are about to delete ALL high scores for ".white(),
            game_name.yellow().bold(),
        ]),
        Line::from(""),
        Line::from("This action CANNOT be undone!".red()),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            "Press ".gray(),
            "Y".green().bold(),
            " to confirm or ".gray(),
            "N".red().bold(),
            " to cancel".gray(),
        ]),
    ];

    let confirmation = Paragraph::new(confirmation_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" ⚠️  Confirm Deletion  ⚠️ ".red().bold())
                .border_style(Style::new().red().bold())
                .style(Style::default().bg(Color::Rgb(30, 10, 10))),
        );

    frame.render_widget(confirmation, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::games::GameRegistry;
    use ratatui::{backend::TestBackend, Terminal};
    use std::time::Duration;

    #[test]
    fn menu_draws_on_any_size_and_selector_glides() {
        let registry = GameRegistry::new();
        let mut menu = MainMenu::new(registry.list_games()).expect("menu");
        for (w, h) in [
            (1, 1),
            (5, 5),
            (30, 10),
            (60, 20),
            (80, 24),
            (120, 40),
            (250, 80),
        ] {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            for _ in 0..3 {
                menu.animate(Duration::from_millis(16));
                term.draw(|f| menu.draw(f)).unwrap();
            }
            // Main → Games → Down → back, redessiné à chaque étape
            for k in [KeyCode::Enter, KeyCode::Enter, KeyCode::Down] {
                menu.handle_key(KeyEvent::from(k));
                menu.animate(Duration::from_millis(100));
                menu.animate(Duration::from_millis(100));
                term.draw(|f| menu.draw(f)).unwrap();
            }
            for _ in 0..2 {
                menu.handle_key(KeyEvent::from(KeyCode::Esc));
                menu.animate(Duration::from_millis(100));
                menu.animate(Duration::from_millis(100));
            }
        }

        // Un appui pendant le fondu entre écrans est ignoré, puis les touches repassent
        menu.handle_key(KeyEvent::from(KeyCode::Enter)); // Main → Games
        assert!(menu.trans > 0.0);
        let before = menu.selected_index;
        menu.handle_key(KeyEvent::from(KeyCode::Down));
        assert_eq!(
            menu.selected_index, before,
            "touche non ignorée pendant le fondu"
        );
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
        assert_eq!(menu.trans, 0.0);
        menu.handle_key(KeyEvent::from(KeyCode::Esc));
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
        assert_eq!(menu.current_menu, MenuState::Main);

        // Le glissement converge vers l'élément sélectionné sans le dépasser
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
        menu.handle_key(KeyEvent::from(KeyCode::Down));
        menu.handle_key(KeyEvent::from(KeyCode::Down));
        menu.animate(Duration::from_millis(16));
        assert!(menu.sel_pos > 0.0 && menu.sel_pos < 2.0, "{}", menu.sel_pos);
        for _ in 0..120 {
            menu.animate(Duration::from_millis(16));
        }
        assert!((menu.sel_pos - 2.0).abs() < 0.01);
        assert_eq!(menu.card_t, 1.0);
    }

    #[test]
    fn slide_mixes_old_and_new_screens_without_black() {
        let registry = GameRegistry::new();
        let mut menu = MainMenu::new(registry.list_games()).expect("menu");
        let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
        term.draw(|f| menu.draw(f)).unwrap();
        let before = term.backend().buffer().clone();
        menu.handle_key(KeyEvent::from(KeyCode::Enter)); // → Games (glissement)
        menu.animate(Duration::from_millis(40));
        term.draw(|f| menu.draw(f)).unwrap();
        let mid = term.backend().buffer().clone();
        // la bordure gauche de l'ancien écran a bougé, et rien n'est devenu noir pur
        assert_ne!(before, mid);
        assert!(mid.content().iter().all(|c| c.bg != Color::Black));
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
        term.draw(|f| menu.draw(f)).unwrap();
        assert!(menu.slide_old.is_none());
    }

    fn press(menu: &mut MainMenu, k: KeyCode) {
        menu.handle_key(KeyEvent::from(k));
        menu.animate(Duration::from_millis(100));
        menu.animate(Duration::from_millis(100));
    }

    #[test]
    fn settings_tabs_adjust_and_persist() {
        let registry = GameRegistry::new();
        let mut menu = MainMenu::new(registry.list_games()).expect("menu");
        menu.animate(Duration::from_millis(100));
        for _ in 0..3 {
            press(&mut menu, KeyCode::Down);
        }
        press(&mut menu, KeyCode::Enter);
        assert_eq!(menu.current_menu, MenuState::Settings);
        assert_eq!(menu.settings_tab, 0);

        // Onglets: Tab, BackTab et touches numériques
        press(&mut menu, KeyCode::Tab);
        assert_eq!(menu.settings_tab, 1);
        press(&mut menu, KeyCode::Char('3'));
        assert_eq!(menu.settings_tab, 2);
        press(&mut menu, KeyCode::BackTab);
        assert_eq!(menu.settings_tab, 1);

        // Curseur de volume: ±10 %, borné à 0..1, sauvegardé
        let before = menu.audio.get_master_volume();
        press(&mut menu, KeyCode::Left);
        assert!((menu.audio.get_master_volume() - (before - 0.1).max(0.0)).abs() < 0.001);
        for _ in 0..12 {
            press(&mut menu, KeyCode::Right);
        }
        assert_eq!(menu.audio.get_master_volume(), 1.0);
        assert_eq!(menu.config_manager.get().audio.master_volume, 1.0);

        // Interrupteur de gameplay: Entrée bascule et la config est enregistrée
        press(&mut menu, KeyCode::Tab);
        assert!(menu.config_manager.get().gameplay.ghost_piece);
        press(&mut menu, KeyCode::Enter);
        assert!(!menu.config_manager.get().gameplay.ghost_piece);
        assert!(!crate::engine::fx::ghost_piece());
        press(&mut menu, KeyCode::Enter);
        assert!(menu.config_manager.get().gameplay.ghost_piece);

        // Choix visuel: Effets Full → Low → (retour) Full
        press(&mut menu, KeyCode::Tab);
        assert_eq!(menu.settings_tab, 0);
        press(&mut menu, KeyCode::Right);
        assert_eq!(
            menu.config_manager.get().visuals.fx_mode,
            crate::config::FxMode::Low
        );
        press(&mut menu, KeyCode::Left);
        assert_eq!(
            menu.config_manager.get().visuals.fx_mode,
            crate::config::FxMode::Full
        );

        // Le fichier de config relu contient les changements
        let reloaded = ConfigManager::new().unwrap();
        assert_eq!(reloaded.get().audio.master_volume, 1.0);
    }
}
