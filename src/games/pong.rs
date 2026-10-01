use crate::audio::{AudioManager, SoundEffect};
use crate::core::{Game, GameAction};
use crate::engine::{
    braille::Braille,
    fx::{self, Shake},
    particles::Particles,
};
use crate::highscores::{GameData, HighScoreManager, Score};
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
        let mut rng = crate::engine::rng::game_rng();
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

    // High scores
    highscore_manager: HighScoreManager,
    start_time: std::time::Instant,
    score_saved: bool,

    // Effets
    particles: Particles,
    shake: Shake,
    trail: Braille,
    prev_ball: (f32, f32), // position avant le dernier tick (interpolation)
    last_stamp: (f32, f32),
    since_tick: f32,
    wall_flash: [(f32, f32); 2], // (intensité, x) du dernier rebond haut / bas
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

            highscore_manager: HighScoreManager::default(),
            start_time: std::time::Instant::now(),
            score_saved: false,

            particles: Particles::new(256, 8.0),
            shake: Shake::default(),
            trail: Braille::new(width as u16, height as u16),
            prev_ball: (width / 2.0, height / 2.0),
            last_stamp: (width / 2.0, height / 2.0),
            since_tick: 0.0,
            wall_flash: [(0.0, 0.0); 2],
        }
    }

    fn start_game(&mut self, mode: GameMode) {
        self.mode = mode;
        self.state = PongState::Playing;
        self.score_player1 = 0;
        self.score_player2 = 0;
        self.score_saved = false;
        self.start_time = std::time::Instant::now();
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
        self.prev_ball = (self.ball.position.x, self.ball.position.y);
        self.since_tick = 0.0;
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
                let (i, y) = if self.ball.velocity.dy > 0.0 {
                    (0, -1.0) // rebond sur le haut
                } else {
                    (1, self.height)
                };
                let x = self.ball.position.x + 0.5;
                self.wall_flash[i] = (1.0, x);
                let c = (120, 235, 255);
                self.particles
                    .burst(x, y, 4, 6.0, c, fx::lerp(c, (0, 0, 0), 0.8));
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
            let mut rng = crate::engine::rng::game_rng();
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
            self.particles.burst(
                self.ball.position.x,
                ball_y + 0.5,
                6,
                7.0,
                (200, 230, 255),
                (40, 80, 200),
            );
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
            self.particles.burst(
                self.ball.position.x + 1.0,
                ball_y + 0.5,
                6,
                7.0,
                (255, 220, 200),
                (200, 50, 40),
            );
            self.audio.play_sound(SoundEffect::PongPaddleHit);
        }
    }

    fn check_scoring(&mut self) {
        // Joueur 1 marque (balle sort à droite)
        if self.ball.position.x >= self.width {
            self.score_player1 += 1;
            self.goal_burst(self.width - 1.0, (110, 170, 255));
            self.audio.play_sound(SoundEffect::PongScore);
            self.check_game_over();
            if self.state == PongState::Playing {
                self.reset_positions();
            }
        }

        // Joueur 2 marque (balle sort à gauche)
        if self.ball.position.x <= 0.0 {
            self.score_player2 += 1;
            self.goal_burst(0.0, (255, 110, 100));
            self.audio.play_sound(SoundEffect::PongScore);
            self.check_game_over();
            if self.state == PongState::Playing {
                self.reset_positions();
            }
        }
    }

    /// Gerbe de particules le long du bord marqué + secousse.
    fn goal_burst(&mut self, x: f32, c: fx::Rgb) {
        const N: usize = 6;
        for i in 0..N {
            let y = self.height * (i as f32 + 0.5) / N as f32;
            self.particles
                .burst(x, y, 8, 9.0, c, fx::lerp(c, (0, 0, 0), 0.85));
        }
        self.shake.kick(1.5);
    }

    fn check_game_over(&mut self) {
        if self.score_player1 >= self.max_score || self.score_player2 >= self.max_score {
            self.state = PongState::GameOver;
            // Arrêter la musique normale et jouer la célébration
            self.audio.stop_music();
            self.audio.play_pong_music_celebration();
            self.music_started = false;

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
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

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // On sauvegarde seulement le score du joueur humain (joueur 1)
        // En mode single player, le score du joueur 1 est ce qui compte
        // En mode 2 joueurs, on peut sauvegarder le meilleur des deux scores
        let player_score = match self.mode {
            GameMode::SinglePlayer => self.score_player1, // Score contre l'IA
            GameMode::TwoPlayer => self.score_player1.max(self.score_player2), // Meilleur score en 2 joueurs
        };

        // Vérifier si c'est un high score
        if self.highscore_manager.is_high_score("pong", player_score) {
            let duration = self.start_time.elapsed().as_secs();

            // Le score de l'adversaire (IA ou joueur 2)
            let opponent_score = match self.mode {
                GameMode::SinglePlayer => self.score_player2, // Score de l'IA
                GameMode::TwoPlayer => self.score_player1.min(self.score_player2), // Score le plus bas
            };

            let game_data = GameData::Pong {
                opponent_score,
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), player_score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("pong", score) {
                self.score_saved = true;
            }
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
                    // Nettoyer l'audio avant de redémarrer
                    self.audio.clear_effects();
                    self.audio.stop_music();
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
            // Téléportation (reset après un but): pas d'interpolation
            let (px, py) = self.prev_ball;
            if (self.ball.position.x - px).hypot(self.ball.position.y - py) > 6.0 {
                self.prev_ball = (self.ball.position.x, self.ball.position.y);
            }
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_pong_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(25) // Très fluide et réactif
    }

    fn frame_time(&self) -> Option<Duration> {
        Some(Duration::from_millis(16))
    }

    fn animate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.particles.update(dt);
        self.shake.update(dt);
        for w in &mut self.wall_flash {
            w.0 = (w.0 - dt / 0.25).max(0.0);
        }
        self.trail.resize(self.width as u16, self.height as u16);

        // Balle interpolée entre deux ticks: affichage fluide, logique inchangée
        self.since_tick += dt;
        let a = (self.since_tick / self.tick_rate().as_secs_f32()).min(1.0);
        let (px, py) = self.prev_ball;
        let b = &self.ball.position;
        let (x, y) = (px + (b.x - px) * a + 0.5, py + (b.y - py) * a + 0.5);
        if (x - self.last_stamp.0).hypot(y - self.last_stamp.1) > 6.0 {
            self.last_stamp = (x, y);
        }
        // Traînée plus longue quand l'échange s'accélère
        let speed = self.ball.velocity.dx.hypot(self.ball.velocity.dy);
        self.trail.decay(dt, 0.04 + 0.06 * speed);
        self.trail
            .line(self.last_stamp.0, self.last_stamp.1, x, y, 1.5);
        self.last_stamp = (x, y);
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

    let (sx, sy) = game.shake.offset();
    let ox = start_x as i32 + sx;
    let oy = start_y as i32 + sy;
    let buf = frame.buffer_mut();

    // Bordures néon haut/bas (collées au terrain), avec flash localisé au rebond
    for (i, (row, ch)) in [(oy - 1, '▄'), (oy + game_height as i32, '▀')]
        .into_iter()
        .enumerate()
    {
        let (f, fxp) = game.wall_flash[i];
        for x in 0..game_width as i32 {
            let near = (1.0 - ((x as f32 - fxp).abs() / 8.0)).max(0.0) * f;
            let c = fx::lerp((30, 110, 140), (200, 255, 255), near);
            fx::put(buf, ox + x, row, ch, c);
        }
    }

    // Ligne centrale en pointillés
    for y in (0..game_height as i32).step_by(2) {
        fx::put(
            buf,
            ox + (game_width / 2) as i32,
            oy + y,
            '┃',
            (40, 100, 120),
        );
    }

    // Paddles avec lueur
    for (p, glow, hi, lo) in [
        (
            &game.player1,
            (80, 140, 255),
            (190, 220, 255),
            (70, 120, 230),
        ),
        (&game.player2, (255, 90, 80), (255, 200, 190), (230, 70, 60)),
    ] {
        let (px, py) = (ox + p.position.x as i32, oy + p.position.y as i32);
        let h = p.height as i32;
        fx::glow(buf, px, py + h / 2, 4, glow, 0.45);
        for i in 0..h {
            let t = (i as f32 / (h - 1).max(1) as f32 - 0.5).abs() * 2.0;
            fx::put(buf, px, py + i, '█', fx::lerp(hi, lo, t));
        }
    }

    // Balle (Braille) + traînée
    game.trail.draw(buf, ox, oy, (140, 245, 255), (20, 70, 110));
    game.particles.draw(buf, (ox, oy));

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
