use crate::audio::{AudioManager, SoundEffect};
use crate::core::{Game, GameAction};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameMode {
    SinglePlayer, // Contre IA
    TwoPlayer,    // 2 joueurs
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PongState {
    Menu,
    Playing,
    GameOver,
}

pub struct Ball {
    position: Position,
    velocity: Velocity,
    #[allow(dead_code)]
    size: f32,
}

impl Ball {
    fn new(width: f32, height: f32) -> Self {
        let mut rng = rand::rng();
        let angle = rng.random_range(-std::f32::consts::PI / 4.0..std::f32::consts::PI / 4.0);
        let speed = 0.8;
        let direction = if rng.random_bool(0.5) { 1.0 } else { -1.0 };

        Self {
            position: Position {
                x: width / 2.0,
                y: height / 2.0,
            },
            velocity: Velocity {
                dx: direction * speed * angle.cos(),
                dy: speed * angle.sin(),
            },
            size: 1.0,
        }
    }

    fn reset(&mut self, width: f32, height: f32) {
        *self = Self::new(width, height);
    }
}

pub struct Paddle {
    position: Position,
    height: f32,
    speed: f32,
}

impl Paddle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            position: Position { x, y },
            height: 4.0,
            speed: 2.5,
        }
    }

    fn move_up(&mut self, _field_height: f32) {
        self.position.y = (self.position.y - self.speed).max(0.0);
    }

    fn move_down(&mut self, field_height: f32) {
        self.position.y = (self.position.y + self.speed).min(field_height - self.height);
    }

    fn get_center(&self) -> f32 {
        self.position.y + self.height / 2.0
    }
}

pub struct PongGame {
    state: PongState,
    mode: GameMode,
    selected_mode: usize, // Pour le menu de sélection de mode

    // Terrain
    width: f32,
    height: f32,

    // Objets du jeu
    ball: Ball,
    player1: Paddle, // Joueur gauche
    player2: Paddle, // Joueur droite ou IA

    // Scores
    score_player1: u32,
    score_player2: u32,
    max_score: u32,

    // IA
    ai_difficulty: f32,     // Entre 0.0 et 1.0
    ai_update_counter: u32, // Compteur pour ralentir l'IA

    // Audio
    audio: AudioManager,
    music_started: bool,
}

impl PongGame {
    pub fn new() -> Self {
        let width = 60.0;
        let height = 20.0;

        Self {
            state: PongState::Menu,
            mode: GameMode::SinglePlayer,
            selected_mode: 0,

            width,
            height,

            ball: Ball::new(width, height),
            player1: Paddle::new(2.0, height / 2.0 - 2.0),
            player2: Paddle::new(width - 4.0, height / 2.0 - 2.0),

            score_player1: 0,
            score_player2: 0,
            max_score: 5,

            ai_difficulty: 0.7, // IA modérément difficile
            ai_update_counter: 0,

            audio: AudioManager::default(),
            music_started: false,
        }
    }

    fn start_game(&mut self, mode: GameMode) {
        self.mode = mode;
        self.state = PongState::Playing;
        self.score_player1 = 0;
        self.score_player2 = 0;
        self.reset_positions();
    }

    fn reset_positions(&mut self) {
        self.ball.reset(self.width, self.height);
        self.player1.position.y = self.height / 2.0 - self.player1.height / 2.0;
        self.player2.position.y = self.height / 2.0 - self.player2.height / 2.0;
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() && self.state == PongState::Playing
        {
            self.audio.play_pong_music();
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started
            && self.audio.is_music_enabled()
            && self.state == PongState::Playing
            && self.audio.is_music_empty()
        {
            // Jouer version rapide si la balle va très vite
            let ball_speed = (self.ball.velocity.dx.powi(2) + self.ball.velocity.dy.powi(2)).sqrt();
            if ball_speed > 1.5 {
                self.audio.play_pong_music_fast();
            } else {
                self.audio.play_pong_music();
            }
        }
    }

    fn update_ball(&mut self) {
        // Sauvegarder l'ancienne position Y pour détecter les collisions avec les murs
        let old_y = self.ball.position.y;

        // Mettre à jour la position
        self.ball.position.x += self.ball.velocity.dx;
        self.ball.position.y += self.ball.velocity.dy;

        // Rebond sur les murs haut et bas
        if self.ball.position.y <= 0.0 || self.ball.position.y >= self.height - 1.0 {
            self.ball.velocity.dy = -self.ball.velocity.dy;
            self.ball.position.y = self.ball.position.y.clamp(0.0, self.height - 1.0);

            // Jouer le son de collision avec le mur seulement si on vient de toucher
            if old_y > 0.0 && old_y < self.height - 1.0 {
                self.audio.play_sound(SoundEffect::PongWallHit);
            }
        }
    }

    fn update_ai(&mut self) {
        if self.mode == GameMode::SinglePlayer {
            // L'IA ne réagit que toutes les 3 frames pour éviter les mouvements épileptiques
            self.ai_update_counter += 1;
            if self.ai_update_counter < 3 {
                return;
            }
            self.ai_update_counter = 0;

            let ball_center_y = self.ball.position.y;
            let paddle_center_y = self.player2.get_center();

            let diff = ball_center_y - paddle_center_y;

            // L'IA n'est pas parfaite, elle a une vitesse limitée et parfois rate
            let mut rng = rand::rng();
            let _reaction_speed = self.ai_difficulty * self.player2.speed;

            // Zone morte élargie pour éviter les mouvements épileptiques
            let dead_zone = 1.5; // Zone morte plus large

            // Ajouter un peu d'imprécision à l'IA
            let error = rng.random_range(-0.3..0.3) * (1.0 - self.ai_difficulty);
            let target_diff = diff + error;

            // Ne bouger que si on est vraiment loin du centre
            if target_diff > dead_zone {
                self.player2.move_down(self.height);
            } else if target_diff < -dead_zone {
                self.player2.move_up(self.height);
            }
        }
    }

    fn check_ball_collision(&mut self) {
        let ball_x = self.ball.position.x;
        let ball_y = self.ball.position.y;

        // Collision avec le paddle gauche (joueur 1)
        if ball_x <= self.player1.position.x + 1.0
            && ball_x >= self.player1.position.x
            && ball_y >= self.player1.position.y
            && ball_y <= self.player1.position.y + self.player1.height
        {
            self.ball.velocity.dx = -self.ball.velocity.dx * 1.05; // Légère accélération

            // Modifier l'angle selon où la balle touche le paddle
            let hit_pos = (ball_y - self.player1.get_center()) / (self.player1.height / 2.0);
            self.ball.velocity.dy += hit_pos * 0.3;

            self.ball.position.x = self.player1.position.x + 1.0;
            self.audio.play_sound(SoundEffect::PongPaddleHit);
        }

        // Collision avec le paddle droit (joueur 2 ou IA)
        if ball_x >= self.player2.position.x - 1.0
            && ball_x <= self.player2.position.x
            && ball_y >= self.player2.position.y
            && ball_y <= self.player2.position.y + self.player2.height
        {
            self.ball.velocity.dx = -self.ball.velocity.dx * 1.05; // Légère accélération

            // Modifier l'angle selon où la balle touche le paddle
            let hit_pos = (ball_y - self.player2.get_center()) / (self.player2.height / 2.0);
            self.ball.velocity.dy += hit_pos * 0.3;

            self.ball.position.x = self.player2.position.x - 1.0;
            self.audio.play_sound(SoundEffect::PongPaddleHit);
        }
    }

    fn check_scoring(&mut self) {
        // Joueur 1 marque (balle sort à droite)
        if self.ball.position.x >= self.width {
            self.score_player1 += 1;
            self.audio.play_sound(SoundEffect::PongScore);
            self.check_game_over();
            if self.state == PongState::Playing {
                self.reset_positions();
            }
        }

        // Joueur 2 marque (balle sort à gauche)
        if self.ball.position.x <= 0.0 {
            self.score_player2 += 1;
            self.audio.play_sound(SoundEffect::PongScore);
            self.check_game_over();
            if self.state == PongState::Playing {
                self.reset_positions();
            }
        }
    }

    fn check_game_over(&mut self) {
        if self.score_player1 >= self.max_score || self.score_player2 >= self.max_score {
            self.state = PongState::GameOver;
            // Arrêter la musique normale et jouer la célébration
            self.audio.stop_music();
            self.audio.play_pong_music_celebration();
            self.music_started = false;
        }
    }

    fn update_dimensions(&mut self, new_width: f32, new_height: f32) {
        if self.width != new_width || self.height != new_height {
            let width_ratio = new_width / self.width;
            let height_ratio = new_height / self.height;

            // Mettre à jour les dimensions
            self.width = new_width;
            self.height = new_height;

            // Ajuster les positions proportionnellement
            self.ball.position.x *= width_ratio;
            self.ball.position.y *= height_ratio;

            self.player1.position.y *= height_ratio;
            self.player2.position.x = new_width - 4.0; // Repositionner à droite
            self.player2.position.y *= height_ratio;
        }
    }
}

impl Game for PongGame {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match self.state {
            PongState::Menu => match key.code {
                KeyCode::Up => {
                    self.selected_mode = if self.selected_mode == 0 { 1 } else { 0 };
                    GameAction::Continue
                }
                KeyCode::Down => {
                    self.selected_mode = if self.selected_mode == 1 { 0 } else { 1 };
                    GameAction::Continue
                }
                KeyCode::Enter => {
                    let mode = if self.selected_mode == 0 {
                        GameMode::SinglePlayer
                    } else {
                        GameMode::TwoPlayer
                    };
                    self.start_game(mode);
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            },
            PongState::Playing => {
                match key.code {
                    // Contrôles joueur 1 (gauche)
                    KeyCode::Char('w') => {
                        self.player1.move_up(self.height);
                        GameAction::Continue
                    }
                    KeyCode::Char('s') => {
                        self.player1.move_down(self.height);
                        GameAction::Continue
                    }
                    // Contrôles joueur 2 (droite) - seulement en mode 2 joueurs
                    KeyCode::Up if self.mode == GameMode::TwoPlayer => {
                        self.player2.move_up(self.height);
                        GameAction::Continue
                    }
                    KeyCode::Down if self.mode == GameMode::TwoPlayer => {
                        self.player2.move_down(self.height);
                        GameAction::Continue
                    }
                    KeyCode::Char('q') => GameAction::Quit,
                    KeyCode::Esc => {
                        self.state = PongState::Menu;
                        self.audio.stop_music();
                        self.music_started = false;
                        GameAction::Continue
                    }
                    // Contrôles audio/musique
                    KeyCode::Char('m') => {
                        self.audio.toggle_music();
                        if self.audio.is_music_enabled() {
                            self.start_music_if_needed();
                        } else {
                            self.music_started = false;
                        }
                        GameAction::Continue
                    }
                    KeyCode::Char('n') => {
                        self.audio.toggle_enabled();
                        GameAction::Continue
                    }
                    _ => GameAction::Continue,
                }
            }
            PongState::GameOver => match key.code {
                KeyCode::Char('r') => {
                    self.start_game(self.mode);
                    GameAction::Continue
                }
                KeyCode::Char('m') => {
                    self.state = PongState::Menu;
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            },
        }
    }

    fn update(&mut self) -> GameAction {
        if self.state == PongState::Playing {
            // Gérer la musique
            self.start_music_if_needed();

            self.update_ball();
            self.update_ai();
            self.check_ball_collision();
            self.check_scoring();
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_pong_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(25) // Très fluide et réactif
    }

    fn cleanup(&mut self) {
        // Nettoyer proprement les ressources audio
        self.audio.shutdown();
    }
}

fn draw_pong_game(frame: &mut ratatui::Frame, game: &mut PongGame) {
    let area = frame.area();

    // Fond sombre élégant
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    match game.state {
        PongState::Menu => draw_mode_selection(frame, area, game),
        PongState::Playing => draw_game_field(frame, area, game),
        PongState::GameOver => draw_game_over(frame, area, game),
    }
}

fn draw_mode_selection(frame: &mut ratatui::Frame, area: Rect, game: &PongGame) {
    let chunks = Layout::vertical([
        Constraint::Length(6), // Header
        Constraint::Min(0),    // Menu
        Constraint::Length(3), // Footer
    ])
    .split(area);

    // Header
    let header_text = vec![
        Line::from(""),
        Line::from(vec![
            "🏓 ".yellow().bold(),
            "PONG".cyan().bold(),
            " 🏓".yellow().bold(),
        ]),
        Line::from("Choose your game mode".magenta()),
        Line::from(""),
    ];

    let header = Paragraph::new(header_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Game Selection ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(header, chunks[0]);

    // Menu options
    let modes = ["🤖 Single Player (vs AI)", "👥 Two Players"];
    let mut menu_text = vec![Line::from("")];

    for (i, mode) in modes.iter().enumerate() {
        let style = if i == game.selected_mode {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if i == game.selected_mode {
            "▶ "
        } else {
            "  "
        };
        menu_text.push(Line::from(vec![
            prefix.yellow().bold(),
            (*mode).fg(style.fg.unwrap_or(Color::White)),
        ]));
        menu_text.push(Line::from(""));
    }

    let menu = Paragraph::new(menu_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Select Mode ".green().bold())
                .border_style(Style::new().green())
                .style(Style::default().bg(Color::Rgb(10, 15, 20))),
        );
    frame.render_widget(menu, chunks[1]);

    // Footer
    let footer_text = vec![Line::from(vec![
        "↑↓".cyan().bold(),
        " Navigate  ".white(),
        "Enter".green().bold(),
        " Select  ".white(),
        "Q".red().bold(),
        " Quit".white(),
    ])];

    let footer = Paragraph::new(footer_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(footer, chunks[2]);
}

fn draw_game_field(frame: &mut ratatui::Frame, area: Rect, game: &mut PongGame) {
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec scores
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer avec contrôles
    ])
    .split(area);

    let game_area = chunks[1];
    let inner_area = game_area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    // Calculer les dimensions du terrain de jeu (utilise la taille disponible avec des limites)
    let field_width = inner_area.width.clamp(40, 120) as f32; // Largeur max 120, min 40
    let field_height = inner_area.height.clamp(15, 30) as f32; // Hauteur max 30, min 15

    // Mettre à jour les dimensions du jeu
    game.update_dimensions(field_width, field_height);

    // === HEADER AVEC SCORES ===
    let mode_text = match game.mode {
        GameMode::SinglePlayer => "vs AI",
        GameMode::TwoPlayer => "2 Players",
    };

    let header_text = vec![
        Line::from(vec![
            "🏓 ".yellow().bold(),
            "PONG ".cyan().bold(),
            format!("({mode_text})").gray(),
        ]),
        Line::from(vec![
            "Player 1: ".blue().bold(),
            format!("{}", game.score_player1).white().bold(),
            "  vs  ".gray(),
            "Player 2: ".red().bold(),
            format!("{}", game.score_player2).white().bold(),
            "  |  ".gray(),
            "First to ".yellow(),
            format!("{}", game.max_score).green().bold(),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Game Status ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(header, chunks[0]);

    // === TERRAIN DE JEU ===
    let game_block = Block::bordered()
        .title(" Playing Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    frame.render_widget(game_block, game_area);

    // Créer une zone centrée pour le terrain de jeu
    let game_width = field_width as u16;
    let game_height = field_height as u16;
    let start_x = inner_area.x + (inner_area.width.saturating_sub(game_width)) / 2;
    let start_y = inner_area.y + (inner_area.height.saturating_sub(game_height)) / 2;

    let playing_area = Rect {
        x: start_x,
        y: start_y,
        width: game_width,
        height: game_height,
    };

    // Dessiner le terrain avec une grille subtile
    for y in 0..game_height {
        for x in 0..game_width {
            let cell_x = playing_area.x + x;
            let cell_y = playing_area.y + y;

            if cell_x < playing_area.x + playing_area.width
                && cell_y < playing_area.y + playing_area.height
            {
                let cell_area = Rect {
                    x: cell_x,
                    y: cell_y,
                    width: 1,
                    height: 1,
                };

                // Ligne centrale en pointillés
                let symbol = if x == (field_width as u16 / 2) && y % 3 == 0 {
                    "┃"
                } else {
                    " "
                };

                let color = if x == (field_width as u16 / 2) && y % 3 == 0 {
                    Color::Rgb(100, 100, 100)
                } else {
                    Color::Rgb(20, 25, 30)
                };

                let cell = Paragraph::new(symbol).style(Style::default().fg(color));
                frame.render_widget(cell, cell_area);
            }
        }
    }

    // Dessiner le paddle gauche (joueur 1)
    for i in 0..(game.player1.height as u16) {
        let paddle_x = playing_area.x + game.player1.position.x as u16;
        let paddle_y = playing_area.y + (game.player1.position.y as u16) + i;

        if paddle_x < playing_area.x + playing_area.width
            && paddle_y < playing_area.y + playing_area.height
        {
            let paddle_area = Rect {
                x: paddle_x,
                y: paddle_y,
                width: 1,
                height: 1,
            };

            let paddle_cell =
                Paragraph::new("█").style(Style::default().fg(Color::LightBlue).bold());
            frame.render_widget(paddle_cell, paddle_area);
        }
    }

    // Dessiner le paddle droit (joueur 2 ou IA)
    for i in 0..(game.player2.height as u16) {
        let paddle_x = playing_area.x + game.player2.position.x as u16;
        let paddle_y = playing_area.y + (game.player2.position.y as u16) + i;

        if paddle_x < playing_area.x + playing_area.width
            && paddle_y < playing_area.y + playing_area.height
        {
            let paddle_area = Rect {
                x: paddle_x,
                y: paddle_y,
                width: 1,
                height: 1,
            };

            let paddle_cell =
                Paragraph::new("█").style(Style::default().fg(Color::LightRed).bold());
            frame.render_widget(paddle_cell, paddle_area);
        }
    }

    // Dessiner la balle
    let ball_x = playing_area.x + game.ball.position.x as u16;
    let ball_y = playing_area.y + game.ball.position.y as u16;

    if ball_x < playing_area.x + playing_area.width && ball_y < playing_area.y + playing_area.height
    {
        let ball_area = Rect {
            x: ball_x,
            y: ball_y,
            width: 1,
            height: 1,
        };

        let ball_cell = Paragraph::new("◉").style(Style::default().fg(Color::Cyan).bold());
        frame.render_widget(ball_cell, ball_area);
    }

    // === FOOTER AVEC CONTRÔLES ===
    let controls = match game.mode {
        GameMode::SinglePlayer => {
            "W/S Move Player 1  •  AI controls Player 2  •  Esc Menu  •  Q Quit"
        }
        GameMode::TwoPlayer => "W/S Player 1  •  ↑↓ Player 2  •  Esc Menu  •  Q Quit",
    };

    let footer_text = vec![Line::from(controls.white())];

    let footer = Paragraph::new(footer_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(footer, chunks[2]);
}

fn draw_game_over(frame: &mut ratatui::Frame, area: Rect, game: &mut PongGame) {
    // D'abord dessiner le terrain en arrière-plan
    draw_game_field(frame, area, game);

    // Puis superposer le popup de game over
    let popup_width = 50.min(area.width);
    let popup_height = 12.min(area.height);
    let popup_area = Rect {
        x: if area.width >= popup_width {
            (area.width - popup_width) / 2
        } else {
            0
        },
        y: if area.height >= popup_height {
            (area.height - popup_height) / 2
        } else {
            0
        },
        width: popup_width,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let winner = if game.score_player1 >= game.max_score {
        "Player 1 Wins!"
    } else {
        match game.mode {
            GameMode::SinglePlayer => "AI Wins!",
            GameMode::TwoPlayer => "Player 2 Wins!",
        }
    };

    let winner_color = if game.score_player1 >= game.max_score {
        Color::Blue
    } else {
        Color::Red
    };

    let game_over_text = vec![
        Line::from(""),
        Line::from("🏆 GAME OVER 🏆".yellow().bold()),
        Line::from(""),
        Line::from(winner.fg(winner_color).bold()),
        Line::from(""),
        Line::from(vec![
            "Final Score: ".white(),
            format!("{}", game.score_player1).blue().bold(),
            " - ".gray(),
            format!("{}", game.score_player2).red().bold(),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            "Press ".gray(),
            "R".green().bold(),
            " to restart, ".gray(),
            "M".yellow().bold(),
            " for menu, or ".gray(),
            "Q".red().bold(),
            " to quit".gray(),
        ]),
    ];

    let popup = Paragraph::new(game_over_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Game Over ".yellow().bold())
                .border_style(Style::new().yellow().bold())
                .style(Style::default().bg(Color::Black)),
        );
    frame.render_widget(popup, popup_area);
}
