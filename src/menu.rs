use crate::audio::AudioManager;
use crate::config::ConfigManager;
use crate::core::{GameAction, GameInfo};
use crate::highscores::HighScoreManager;
use crate::music::{
    breakout::BREAKOUT_MUSIC, gameoflife::GAMEOFLIFE_MUSIC, minesweeper::MINESWEEPER_MUSIC,
    pong::PONG_MUSIC, snake::SNAKE_MUSIC, tetris::TETRIS_MUSIC, GameMusic, _2048::GAME2048_MUSIC,
};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
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
    MusicPlayer,
    Settings,
    AudioSettings,
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
}

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
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> GameAction {
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
                } else if self.current_menu == MenuState::AudioSettings {
                    self.decrease_audio_setting();
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                }
                GameAction::Continue
            }
            KeyCode::Right => {
                if self.current_menu == MenuState::MusicPlayer {
                    self.next_variant();
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
                } else if self.current_menu == MenuState::AudioSettings {
                    self.increase_audio_setting();
                    self.audio.play_sound(crate::audio::SoundEffect::MenuSelect);
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
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => 3,
            MenuState::AudioSettings => 5, // 5 paramètres audio
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
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => 3,
            MenuState::AudioSettings => 5, // 5 paramètres audio
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
                match self.selected_index {
                    0 => {
                        // Audio Settings
                        self.navigate_to(MenuState::AudioSettings);
                    }
                    _ => {
                        self.go_back();
                    }
                }
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
            MenuState::AudioSettings | MenuState::About => {
                self.go_back();
                GameAction::Continue
            }
        }
    }

    /// Navigue vers un nouveau menu en sauvegardant l'état actuel dans la pile
    fn navigate_to(&mut self, new_menu: MenuState) {
        // Sauvegarder le menu actuel dans la pile
        self.menu_history.push(self.current_menu.clone());
        // Passer au nouveau menu
        self.current_menu = new_menu;
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    fn go_back(&mut self) {
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

    fn increase_audio_setting(&mut self) {
        match self.selected_index {
            0 => {
                // Master volume
                let current = self.audio.get_master_volume();
                let new_volume = (current + 0.1).min(1.0);
                self.audio.set_master_volume(new_volume);
            }
            1 => {
                // Effects volume
                let current = self.audio.get_volume();
                let new_volume = (current + 0.1).min(1.0);
                self.audio.set_volume(new_volume);
            }
            2 => {
                // Music volume
                let current = self.audio.get_music_volume();
                let new_volume = (current + 0.1).min(1.0);
                self.audio.set_music_volume(new_volume);
            }
            3 => {
                // Audio enabled - toggle on
                self.audio.set_enabled(true);
            }
            4 => {
                // Music enabled - toggle on
                self.audio.set_music_enabled(true);
            }
            _ => {}
        }
        // Sauvegarder la configuration après modification
        self.save_audio_config();
    }

    fn decrease_audio_setting(&mut self) {
        match self.selected_index {
            0 => {
                // Master volume
                let current = self.audio.get_master_volume();
                let new_volume = (current - 0.1).max(0.0);
                self.audio.set_master_volume(new_volume);
            }
            1 => {
                // Effects volume
                let current = self.audio.get_volume();
                let new_volume = (current - 0.1).max(0.0);
                self.audio.set_volume(new_volume);
            }
            2 => {
                // Music volume
                let current = self.audio.get_music_volume();
                let new_volume = (current - 0.1).max(0.0);
                self.audio.set_music_volume(new_volume);
            }
            3 => {
                // Audio enabled - toggle off
                self.audio.set_enabled(false);
            }
            4 => {
                // Music enabled - toggle off
                self.audio.set_music_enabled(false);
            }
            _ => {}
        }
        // Sauvegarder la configuration après modification
        self.save_audio_config();
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
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header
        Constraint::Min(0),    // Zone principale
        Constraint::Length(3), // Footer
    ])
    .split(area);

    // === HEADER ===
    let title = match &app.current_menu {
        MenuState::Main => "TERMPLAY",
        MenuState::Games => "GAMES",
        MenuState::HighScores => "HIGH SCORES",
        MenuState::HighScoresDetail(_) => "LEADERBOARD",
        MenuState::MusicPlayer => "MUSIC PLAYER",
        MenuState::Settings => "SETTINGS",
        MenuState::AudioSettings => "AUDIO SETTINGS",
        MenuState::About => "ABOUT",
    };

    let subtitle = match &app.current_menu {
        MenuState::Main => "Terminal Mini-Games Collection".to_string(),
        MenuState::Games => "Choose your adventure".to_string(),
        MenuState::HighScores => "Best scores and achievements".to_string(),
        MenuState::HighScoresDetail(game_name) => format!("Top scores for {game_name}"),
        MenuState::MusicPlayer => "Listen to game soundtracks".to_string(),
        MenuState::Settings => "Configure your experience".to_string(),
        MenuState::AudioSettings => "Adjust audio and music settings".to_string(),
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

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Game Status ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(header, chunks[0]);

    // === ZONE PRINCIPALE ===
    match &app.current_menu {
        MenuState::Main => draw_main_options(frame, chunks[1], app),
        MenuState::Games => draw_games_menu(frame, chunks[1], app),
        MenuState::HighScores => draw_highscores_menu(frame, chunks[1], app),
        MenuState::HighScoresDetail(game_name) => {
            let game_name_clone = game_name.clone();
            draw_highscores_detail(frame, chunks[1], app, &game_name_clone)
        }
        MenuState::MusicPlayer => draw_music_player(frame, chunks[1], app),
        MenuState::Settings => draw_settings_menu(frame, chunks[1], app),
        MenuState::AudioSettings => draw_audio_settings_menu(frame, chunks[1], app),
        MenuState::About => draw_about_menu(frame, chunks[1]),
    }

    // === FOOTER ===
    let controls = match app.current_menu {
        MenuState::Main => "Arrow Keys Move • Enter Select • Q Quit",
        MenuState::MusicPlayer => {
            "↑↓ Select Track • ←→ Change Variant • Space/Enter Play • S Stop • Esc/Q Back"
        }
        MenuState::AudioSettings => "↑↓ Select Setting • ←→ Adjust Value • Esc/Q Back",
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
}

fn draw_main_options(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let items: Vec<ListItem> = app
        .main_options
        .iter()
        .map(|option| {
            let content = vec![Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&option.title, Style::default().fg(Color::White).bold()),
                Span::styled("  -  ", Style::default().fg(Color::Gray)),
                Span::styled(&option.description, Style::default().fg(Color::LightBlue)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Main Menu ".white().bold())
                .border_style(Style::new().green())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 100, 200))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_games_menu(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let items: Vec<ListItem> = app
        .games_list
        .iter()
        .map(|game| {
            let icon = match game.name.as_str() {
                "snake" => "🐍",
                "tetris" => "🧩",
                "pong" => "🏓",
                "2048" => "🔢",
                "Minesweeper" => "💣",
                "Breakout" => "🧱",
                "Game of Life" => "🧬",
                _ => "🎮",
            };

            let content = vec![Line::from(vec![
                Span::styled(
                    format!("  {icon} "),
                    Style::default().fg(Color::Green).bold(),
                ),
                Span::styled(
                    game.name.to_uppercase(),
                    Style::default().fg(Color::White).bold(),
                ),
                Span::styled("  -  ", Style::default().fg(Color::Gray)),
                Span::styled(&game.description, Style::default().fg(Color::LightBlue)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Available Games ".green().bold())
                .border_style(Style::new().green())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 50))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_settings_menu(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    let settings_options = [
        "🔊 Audio Settings",
        "🎨 Graphics Settings (Coming soon)",
        "⌨️ Controls Settings (Coming soon)",
    ];

    let items: Vec<ListItem> = settings_options
        .iter()
        .map(|option| {
            let content = vec![Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(*option, Style::default().fg(Color::White).bold()),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Settings Menu ".yellow().bold())
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(200, 150, 0))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_audio_settings_menu(frame: &mut Frame, area: Rect, app: &mut MainMenu) {
    // Créer les options de settings audio avec leurs valeurs actuelles
    let master_volume = app.audio.get_master_volume();
    let volume = app.audio.get_volume();
    let music_volume = app.audio.get_music_volume();
    let audio_enabled = app.audio.is_enabled();
    let music_enabled = app.audio.is_music_enabled();

    // Helper pour créer une barre de volume visuelle
    let create_volume_bar = |value: f32| -> String {
        let filled = (value * 10.0) as usize;
        let empty = 10 - filled;
        format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            (value * 100.0) as u8
        )
    };

    let audio_settings = [
        format!("🎚️ Master Volume     {}", create_volume_bar(master_volume)),
        format!("🔊 Effects Volume    {}", create_volume_bar(volume)),
        format!("🎵 Music Volume      {}", create_volume_bar(music_volume)),
        format!(
            "📢 Audio Enabled     [{}] {}",
            if audio_enabled { "✓" } else { "✗" },
            if audio_enabled { "ON" } else { "OFF" }
        ),
        format!(
            "🎶 Music Enabled     [{}] {}",
            if music_enabled { "✓" } else { "✗" },
            if music_enabled { "ON" } else { "OFF" }
        ),
    ];

    let items: Vec<ListItem> = audio_settings
        .iter()
        .map(|setting| {
            let content = vec![Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(setting, Style::default().fg(Color::White).bold()),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Audio Settings ".cyan().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 200))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_about_menu(frame: &mut Frame, area: Rect) {
    let about_text = vec![
        Line::from(""),
        Line::from("🎮 TermPlay v0.1.0".cyan().bold()),
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
