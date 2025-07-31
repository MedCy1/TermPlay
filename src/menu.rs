use crate::core::{GameAction, GameInfo};
use crate::audio::AudioManager;
use crate::music::{GameMusic, tetris::TETRIS_MUSIC, snake::SNAKE_MUSIC, pong::PONG_MUSIC, _2048::GAME2048_MUSIC};
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
    main_options: Vec<MenuOption>,
    games_list: Vec<GameInfo>,
    selected_index: usize,
    list_state: ListState,
    audio: AudioManager,
    music_tracks: Vec<MusicTrack>,
    current_playing: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct MusicTrack {
    pub name: String,
    pub variants: Vec<String>, // normal, fast, celebration
}

impl MainMenu {
    pub fn new(games: Vec<&GameInfo>) -> Self {
        let main_options = vec![
            MenuOption {
                title: "🎮 Games".to_string(),
                description: "Play exciting terminal games".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::Games),
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
                variants: vec!["Normal".to_string(), "Fast".to_string(), "Celebration".to_string()],
            },
            MusicTrack {
                name: SNAKE_MUSIC.name().to_string(),
                variants: vec!["Normal".to_string(), "Fast".to_string()],
            },
            MusicTrack {
                name: PONG_MUSIC.name().to_string(),
                variants: vec!["Normal".to_string(), "Fast".to_string(), "Celebration".to_string()],
            },
            MusicTrack {
                name: GAME2048_MUSIC.name().to_string(),
                variants: vec!["Normal".to_string(), "Fast".to_string(), "Celebration".to_string()],
            },
        ];

        let audio = AudioManager::default();
        // Activer la musique par défaut pour le music player
        audio.set_music_enabled(true);
        audio.set_enabled(true);

        Self {
            current_menu: MenuState::Main,
            main_options,
            games_list: games.into_iter().cloned().collect(),
            selected_index: 0,
            list_state,
            audio,
            music_tracks,
            current_playing: None,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match key.code {
            KeyCode::Char('q') => {
                if self.current_menu == MenuState::Main {
                    GameAction::Quit
                } else {
                    // Arrêter la musique si on quitte le music player
                    if self.current_menu == MenuState::MusicPlayer {
                        self.audio.stop_music();
                        self.current_playing = None;
                    }
                    self.go_back();
                    GameAction::Continue
                }
            }
            KeyCode::Esc => {
                if self.current_menu != MenuState::Main {
                    self.go_back();
                }
                GameAction::Continue
            }
            KeyCode::Down => {
                self.next_item();
                GameAction::Continue
            }
            KeyCode::Up => {
                self.previous_item();
                GameAction::Continue
            }
            KeyCode::Enter => self.select_current_item(),
            KeyCode::Char(' ') => {
                if self.current_menu == MenuState::MusicPlayer {
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
        let max_items = match self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => 3,
            MenuState::About => 1,
        };

        if max_items == 0 {
            return;
        }

        self.selected_index = (self.selected_index + 1) % max_items;
        self.list_state.select(Some(self.selected_index));
    }

    fn previous_item(&mut self) {
        let max_items = match self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::MusicPlayer => self.music_tracks.len(),
            MenuState::Settings => 3,
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
                            self.current_menu = menu_state.clone();
                            self.selected_index = 0;
                            self.list_state.select(Some(0));
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
            MenuState::Settings | MenuState::About => {
                self.go_back();
                GameAction::Continue
            }
        }
    }

    fn go_back(&mut self) {
        self.current_menu = MenuState::Main;
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }
    
    fn play_selected_music(&mut self) {
        if let Some(track) = self.music_tracks.get(self.selected_index) {
            self.audio.stop_music(); // Arrêter toute musique en cours
            
            // S'assurer que l'audio est activé
            if !self.audio.is_enabled() {
                self.audio.set_enabled(true);
            }
            if !self.audio.is_music_enabled() {
                self.audio.set_music_enabled(true);
            }
            
            // Jouer la musique sélectionnée
            match track.name.as_str() {
                "Tetris (Korobeiniki)" => {
                    self.audio.play_tetris_music();
                }
                "Snake Ambient" => {
                    self.audio.play_snake_music();
                }
                "Pong Retro Electronic" => {
                    self.audio.play_pong_music();
                }
                "2048 Zen Mode" => {
                    self.audio.play_2048_music();
                }
                _ => {}
            }
            
            self.current_playing = Some(self.selected_index);
        }
    }

    pub fn get_selected_game(&self) -> Option<&str> {
        if self.current_menu == MenuState::Games {
            self.games_list.get(self.selected_index).map(|g| g.name.as_str())
        } else {
            None
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        draw_main_menu(frame, self);
    }
    
    pub fn update(&mut self) {
        // Gérer la boucle de musique si on est dans le music player
        if self.current_menu == MenuState::MusicPlayer && self.current_playing.is_some() {
            if self.audio.is_music_enabled() && self.audio.is_music_empty() {
                // Relancer la musique si elle est finie
                self.play_selected_music();
            }
        }
    }
}

fn draw_main_menu(frame: &mut Frame, app: &mut MainMenu) {
    let area = frame.area();

    // Fond sombre élégant
    let background = Block::new()
        .style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // Layout simple et propre
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header
        Constraint::Min(0),    // Zone principale
        Constraint::Length(3), // Footer
    ]).split(area);

    // === HEADER ===
    let title = match app.current_menu {
        MenuState::Main => "TERMPLAY",
        MenuState::Games => "GAMES",
        MenuState::MusicPlayer => "MUSIC PLAYER",
        MenuState::Settings => "SETTINGS",
        MenuState::About => "ABOUT",
    };

    let subtitle = match app.current_menu {
        MenuState::Main => "Terminal Mini-Games Collection",
        MenuState::Games => "Choose your adventure",
        MenuState::MusicPlayer => "Listen to game soundtracks",
        MenuState::Settings => "Configure your experience",
        MenuState::About => "Information about TermPlay",
    };

    let header_text = vec![
        Line::from(vec![
            "🎮 ".cyan().bold(),
            title.yellow().bold(),
            " 🎮".cyan().bold(),
        ]),
        Line::from(subtitle.magenta()),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Game Status ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(25, 35, 45)))
        );
    frame.render_widget(header, chunks[0]);

    // === ZONE PRINCIPALE ===
    match app.current_menu {
        MenuState::Main => draw_main_options(frame, chunks[1], app),
        MenuState::Games => draw_games_menu(frame, chunks[1], app),
        MenuState::MusicPlayer => draw_music_player(frame, chunks[1], app),
        MenuState::Settings => draw_settings_menu(frame, chunks[1]),
        MenuState::About => draw_about_menu(frame, chunks[1]),
    }

    // === FOOTER ===
    let controls = match app.current_menu {
        MenuState::Main => "Arrow Keys Move • Enter Select • Q Quit",
        MenuState::MusicPlayer => "Arrow Keys Move • Space/Enter Play • S Stop • Esc/Q Back",
        _ => "Arrow Keys Move • Enter Select • Esc/Q Back",
    };

    let footer_text = vec![
        Line::from(vec![
            "Controls: ".gray(),
            controls.white().bold(),
        ]),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45)))
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
                .style(Style::default().bg(Color::Rgb(10, 15, 20)))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 100, 200))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
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
                _ => "🎮",
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(Color::Green).bold()),
                Span::styled(game.name.to_uppercase(), Style::default().fg(Color::White).bold()),
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
                .style(Style::default().bg(Color::Rgb(10, 15, 20)))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 50))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_settings_menu(frame: &mut Frame, area: Rect) {
    let settings_text = vec![
        Line::from(""),
        Line::from("⚙️ Settings Menu".yellow().bold()),
        Line::from(""),
        Line::from("🎨 Graphics Settings"),
        Line::from("🔊 Audio Settings"),
        Line::from("⌨️ Controls Settings"),
        Line::from(""),
        Line::from("(Coming soon...)".gray().italic()),
    ];

    let settings = Paragraph::new(settings_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Settings ".yellow().bold())
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(10, 15, 20)))
        );
    frame.render_widget(settings, area);
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
                .style(Style::default().bg(Color::Rgb(10, 15, 20)))
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

            let content = vec![Line::from(vec![
                Span::styled(format!("  {} ", status), Style::default().fg(Color::Green).bold()),
                Span::styled(&track.name, Style::default().fg(Color::White).bold()),
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
                .style(Style::default().bg(Color::Rgb(10, 15, 20)))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(100, 0, 150))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}