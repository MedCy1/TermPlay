use crate::audio::{AudioManager, SoundEffect};
use crate::core::{Game, GameAction};
use crate::engine::{
    braille::Braille,
    fx::{self, Rgb, Shake},
    particles::Particles,
};
use crate::highscores::{GameData, HighScoreManager, Score};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};
use std::time::Duration;

const FIELD_WIDTH: u16 = 60;
const FIELD_HEIGHT: u16 = 20;
const PADDLE_WIDTH: u16 = 10;
const PADDLE_HEIGHT: u16 = 1;
const BRICK_ROWS: usize = 6;
const BRICK_COLS: usize = 12;
const BRICK_WIDTH: u16 = 4;
const BRICK_HEIGHT: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, Copy)]
pub struct Ball {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            dx: 0.8,
            dy: -0.6,
        }
    }

    fn update(&mut self) {
        self.x += self.dx;
        self.y += self.dy;
    }

    fn bounce_x(&mut self) {
        self.dx = -self.dx;
    }

    fn bounce_y(&mut self) {
        self.dy = -self.dy;
    }

    fn reset(&mut self, paddle_x: f32) {
        self.x = paddle_x + PADDLE_WIDTH as f32 / 2.0;
        self.y = FIELD_HEIGHT as f32 - 4.0;
        self.dx = 0.8;
        self.dy = -0.6;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Paddle {
    x: f32,
    y: f32,
}

impl Paddle {
    fn new() -> Self {
        Self {
            x: (FIELD_WIDTH - PADDLE_WIDTH) as f32 / 2.0,
            y: FIELD_HEIGHT as f32 - 2.0,
        }
    }

    fn move_left(&mut self) {
        if self.x > 0.0 {
            self.x = (self.x - 2.0).max(0.0);
        }
    }

    fn move_right(&mut self) {
        if self.x < (FIELD_WIDTH - PADDLE_WIDTH) as f32 {
            self.x = (self.x + 2.0).min((FIELD_WIDTH - PADDLE_WIDTH) as f32);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brick {
    x: u16,
    y: u16,
    destroyed: bool,
    color: Rgb,
}

impl Brick {
    fn new(x: u16, y: u16, row: usize) -> Self {
        let color = match row {
            0 => (235, 64, 64),
            1 => (240, 200, 60),
            2 => (80, 210, 90),
            3 => (70, 210, 220),
            4 => (80, 120, 240),
            _ => (210, 90, 220),
        };

        Self {
            x,
            y,
            destroyed: false,
            color,
        }
    }
}

pub struct BreakoutGame {
    state: GameState,
    ball: Ball,
    paddle: Paddle,
    bricks: [[Brick; BRICK_COLS]; BRICK_ROWS],
    score: u32,
    lives: u32,
    ball_stuck: bool,

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
    fade: f32,
    paddle_flash: f32,
    brick_flash: [[f32; BRICK_COLS]; BRICK_ROWS],
    prev_ball: (f32, f32), // position avant le dernier tick (interpolation)
    last_stamp: (f32, f32),
    since_tick: f32,
}

impl BreakoutGame {
    pub fn new() -> Self {
        let paddle = Paddle::new();
        let ball = Ball::new(paddle.x + PADDLE_WIDTH as f32 / 2.0, paddle.y - 1.0);

        let mut bricks = [[Brick::new(0, 0, 0); BRICK_COLS]; BRICK_ROWS];
        for (row, brick_row) in bricks.iter_mut().enumerate().take(BRICK_ROWS) {
            for (col, brick) in brick_row.iter_mut().enumerate().take(BRICK_COLS) {
                let x = 1 + col as u16 * (BRICK_WIDTH + 1);
                let y = 2 + row as u16 * (BRICK_HEIGHT + 1);
                *brick = Brick::new(x, y, row);
            }
        }

        Self {
            state: GameState::Playing,
            ball,
            paddle,
            bricks,
            score: 0,
            lives: 3,
            ball_stuck: true,

            audio: AudioManager::default(),
            music_started: false,

            highscore_manager: HighScoreManager::default(),
            start_time: std::time::Instant::now(),
            score_saved: false,

            particles: Particles::new(256, 25.0),
            shake: Shake::default(),
            trail: Braille::new(FIELD_WIDTH, FIELD_HEIGHT),
            fade: 0.0,
            paddle_flash: 0.0,
            brick_flash: [[0.0; BRICK_COLS]; BRICK_ROWS],
            prev_ball: (ball.x, ball.y),
            last_stamp: (ball.x, ball.y),
            since_tick: 0.0,
        }
    }

    fn launch_ball(&mut self) {
        if self.ball_stuck {
            self.ball_stuck = false;
        }
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() && self.state == GameState::Playing
        {
            // Compter les briques restantes pour choisir la musique
            let remaining_bricks = self.count_remaining_bricks();
            let total_bricks = (BRICK_ROWS * BRICK_COLS) as u32;
            let completion_ratio = 1.0 - (remaining_bricks as f32 / total_bricks as f32);

            if completion_ratio > 0.7 {
                self.audio.play_breakout_music_fast(); // Version intense pour fin de partie
            } else {
                self.audio.play_breakout_music(); // Version arcade normale
            }
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started
            && self.audio.is_music_enabled()
            && self.state == GameState::Playing
            && self.audio.is_music_empty()
        {
            let remaining_bricks = self.count_remaining_bricks();
            let total_bricks = (BRICK_ROWS * BRICK_COLS) as u32;
            let completion_ratio = 1.0 - (remaining_bricks as f32 / total_bricks as f32);

            if completion_ratio > 0.7 {
                self.audio.play_breakout_music_fast();
            } else {
                self.audio.play_breakout_music();
            }
        }
    }

    fn count_remaining_bricks(&self) -> u32 {
        let mut count = 0;
        for row in &self.bricks {
            for brick in row {
                if !brick.destroyed {
                    count += 1;
                }
            }
        }
        count
    }

    fn wall_spark(&mut self, x: f32) {
        let c = (150, 200, 255);
        self.particles
            .burst(x, self.ball.y + 0.5, 5, 6.0, c, fx::lerp(c, (0, 0, 0), 0.8));
    }

    fn check_collisions(&mut self) {
        // Collision avec les murs
        if self.ball.x <= 0.0 {
            self.ball.x = 0.0;
            self.ball.bounce_x();
            // Son de collision avec les murs (réutilise le son Pong)
            self.audio.play_sound(SoundEffect::PongWallHit);
            self.wall_spark(0.0);
        }
        if self.ball.x >= FIELD_WIDTH as f32 - 1.0 {
            self.ball.x = FIELD_WIDTH as f32 - 1.0;
            self.ball.bounce_x();
            self.audio.play_sound(SoundEffect::PongWallHit);
            self.wall_spark(FIELD_WIDTH as f32);
        }
        if self.ball.y <= 0.0 {
            self.ball.y = 0.0;
            self.ball.bounce_y();
            self.audio.play_sound(SoundEffect::PongWallHit);
            let c = (150, 200, 255);
            self.particles.burst(
                self.ball.x + 0.5,
                0.0,
                5,
                6.0,
                c,
                fx::lerp(c, (0, 0, 0), 0.8),
            );
        }

        // Collision avec la raquette
        if self.ball.y >= self.paddle.y - 1.0
            && self.ball.y <= self.paddle.y + PADDLE_HEIGHT as f32
            && self.ball.x >= self.paddle.x
            && self.ball.x <= self.paddle.x + PADDLE_WIDTH as f32
        {
            self.ball.y = self.paddle.y - 1.0;

            // Ajuster la direction en fonction de la position sur la raquette
            let hit_pos = (self.ball.x - self.paddle.x) / PADDLE_WIDTH as f32;
            let angle_factor = (hit_pos - 0.5) * 2.0; // -1 à 1
            self.ball.dx = angle_factor * 1.2;
            self.ball.dy = -self.ball.dy.abs(); // Toujours vers le haut

            // Son de collision avec la raquette
            self.audio.play_sound(SoundEffect::BreakoutPaddleHit);
            self.paddle_flash = 1.0;
            self.particles.spray(
                self.ball.x + 0.5,
                self.paddle.y,
                16,
                9.0,
                (255, 255, 230),
                (60, 100, 255),
                5.0,
            );
        }

        // Collision avec les briques
        let ball_x = self.ball.x as u16;
        let ball_y = self.ball.y as u16;

        for (r, row) in self.bricks.iter_mut().enumerate() {
            for (c, brick) in row.iter_mut().enumerate() {
                if brick.destroyed {
                    continue;
                }

                // Vérifier collision avec la brique
                if ball_x >= brick.x
                    && ball_x < brick.x + BRICK_WIDTH
                    && ball_y >= brick.y
                    && ball_y < brick.y + BRICK_HEIGHT
                {
                    brick.destroyed = true;
                    if brick.y <= 4 {
                        // Rangées du haut (les plus rémunératrices): micro-shake bref
                        self.shake.kick(0.4);
                    }
                    self.brick_flash[r][c] = 1.0;
                    let (bx, by) = (
                        brick.x as f32 + BRICK_WIDTH as f32 / 2.0,
                        brick.y as f32 + 0.5,
                    );
                    let dark = fx::lerp(brick.color, (0, 0, 0), 0.8);
                    self.particles.burst(bx, by, 22, 10.0, brick.color, dark);
                    self.particles
                        .spray(bx, by, 8, 6.0, (255, 255, 255), brick.color, 4.0);
                    self.score += 10;
                    self.ball.bounce_y();

                    // Son de destruction de brique
                    self.audio.play_sound(SoundEffect::BreakoutBrickHit);
                    break;
                }
            }
        }

        // Vérifier si la balle tombe en bas
        if self.ball.y >= FIELD_HEIGHT as f32 {
            self.lives -= 1;
            self.shake.kick(2.0);
            self.particles.burst(
                self.ball.x + 0.5,
                FIELD_HEIGHT as f32 - 1.0,
                30,
                12.0,
                (255, 120, 60),
                (60, 0, 0),
            );
            if self.lives == 0 {
                self.state = GameState::GameOver;
                // Son de game over
                self.audio.play_sound(SoundEffect::BreakoutGameOver);

                // Sauvegarder le score si c'est un high score et pas encore sauvé
                self.save_high_score_if_needed();
            } else {
                self.ball.reset(self.paddle.x);
                self.ball_stuck = true;
            }
        }

        // Vérifier la victoire
        if self.all_bricks_destroyed() {
            self.shake.kick(0.4);
            self.state = GameState::Victory;
            // Musique de victoire
            self.audio.stop_music();
            self.audio.play_breakout_music_celebration();
            self.music_started = false;

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
        }
    }

    fn all_bricks_destroyed(&self) -> bool {
        for row in &self.bricks {
            for brick in row {
                if !brick.destroyed {
                    return false;
                }
            }
        }
        true
    }

    fn update_ball(&mut self) {
        self.prev_ball = (self.ball.x, self.ball.y);
        self.since_tick = 0.0;
        if self.ball_stuck {
            // La balle suit la raquette
            self.ball.x = self.paddle.x + PADDLE_WIDTH as f32 / 2.0;
        } else {
            self.ball.update();
            self.check_collisions();
        }
        // Téléportation (reset après une vie perdue): pas d'interpolation ni de traînée
        let (px, py) = self.prev_ball;
        if (self.ball.x - px).hypot(self.ball.y - py) > 3.0 {
            self.prev_ball = (self.ball.x, self.ball.y);
        }
    }

    fn restart(&mut self) {
        let paddle = Paddle::new();
        let ball = Ball::new(paddle.x + PADDLE_WIDTH as f32 / 2.0, paddle.y - 1.0);

        let mut bricks = [[Brick::new(0, 0, 0); BRICK_COLS]; BRICK_ROWS];
        for (row, brick_row) in bricks.iter_mut().enumerate().take(BRICK_ROWS) {
            for (col, brick) in brick_row.iter_mut().enumerate().take(BRICK_COLS) {
                let x = 1 + col as u16 * (BRICK_WIDTH + 1);
                let y = 2 + row as u16 * (BRICK_HEIGHT + 1);
                *brick = Brick::new(x, y, row);
            }
        }

        self.state = GameState::Playing;
        self.ball = ball;
        self.paddle = paddle;
        self.bricks = bricks;
        self.score = 0;
        self.lives = 3;
        self.ball_stuck = true;
        self.score_saved = false;
        self.start_time = std::time::Instant::now();

        self.audio.stop_music();
        self.music_started = false;
    }

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // Vérifier si c'est un high score
        if self.highscore_manager.is_high_score("breakout", self.score) {
            let duration = self.start_time.elapsed().as_secs();

            // Calculer les briques détruites
            let total_bricks = (BRICK_ROWS * BRICK_COLS) as u32;
            let remaining_bricks = self.count_remaining_bricks();
            let bricks_broken = total_bricks - remaining_bricks;

            // Calculer un niveau basé sur les briques détruites
            let level = (bricks_broken / 12).max(1); // Niveau augmente tous les 12 briques

            let game_data = GameData::Breakout {
                level,
                bricks_broken,
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), self.score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("breakout", score) {
                self.score_saved = true;
            }
        }
    }
}

impl Game for BreakoutGame {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match self.state {
            GameState::Playing => match key.code {
                KeyCode::Left | KeyCode::Char('a') => {
                    self.paddle.move_left();
                    GameAction::Continue
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.paddle.move_right();
                    GameAction::Continue
                }
                KeyCode::Char(' ') => {
                    self.launch_ball();
                    GameAction::Continue
                }
                KeyCode::Char('p') => {
                    self.state = GameState::Paused;
                    GameAction::Continue
                }
                KeyCode::Char('r') => {
                    // Nettoyer l'audio avant de redémarrer
                    self.audio.clear_effects();
                    self.audio.stop_music();
                    self.restart();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                KeyCode::Char('m') => {
                    self.audio.toggle_music();
                    GameAction::Continue
                }
                KeyCode::Char('n') => {
                    self.audio.toggle_enabled();
                    GameAction::Continue
                }
                _ => GameAction::Continue,
            },
            GameState::Paused => match key.code {
                KeyCode::Char('p') => {
                    self.state = GameState::Playing;
                    GameAction::Continue
                }
                KeyCode::Char('r') => {
                    // Nettoyer l'audio avant de redémarrer
                    self.audio.clear_effects();
                    self.audio.stop_music();
                    self.restart();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                KeyCode::Char('m') => {
                    self.audio.toggle_music();
                    GameAction::Continue
                }
                KeyCode::Char('n') => {
                    self.audio.toggle_enabled();
                    GameAction::Continue
                }
                _ => GameAction::Continue,
            },
            GameState::GameOver | GameState::Victory => match key.code {
                KeyCode::Char('r') => {
                    // Nettoyer l'audio avant de redémarrer
                    self.audio.clear_effects();
                    self.audio.stop_music();
                    self.restart();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                KeyCode::Char('m') => {
                    self.audio.toggle_music();
                    GameAction::Continue
                }
                KeyCode::Char('n') => {
                    self.audio.toggle_enabled();
                    GameAction::Continue
                }
                _ => GameAction::Continue,
            },
        }
    }

    fn update(&mut self) -> GameAction {
        if self.state == GameState::Playing {
            self.start_music_if_needed();
            self.update_ball();
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_breakout_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(50)
    }

    fn frame_time(&self) -> Option<Duration> {
        Some(Duration::from_millis(16))
    }

    fn animate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.particles.update(dt);
        self.shake.update(dt);
        self.paddle_flash = (self.paddle_flash - dt / 0.25).max(0.0);
        for f in self.brick_flash.iter_mut().flatten() {
            *f = (*f - dt / 0.15).max(0.0);
        }
        if self.state == GameState::GameOver {
            self.fade = (self.fade + dt / 0.8).min(1.0);
        } else {
            self.fade = 0.0;
        }

        // Balle interpolée entre deux ticks: affichage fluide, logique inchangée
        self.since_tick += dt;
        let a = (self.since_tick / self.tick_rate().as_secs_f32()).min(1.0);
        let (px, py) = self.prev_ball;
        let (x, y) = (
            px + (self.ball.x - px) * a + 0.5,
            py + (self.ball.y - py) * a + 0.5,
        );
        if (x - self.last_stamp.0).hypot(y - self.last_stamp.1) > 3.0 {
            self.last_stamp = (x, y);
        }
        self.trail.decay(dt, 0.28);
        self.trail
            .line(self.last_stamp.0, self.last_stamp.1, x, y, 1.5);
        self.last_stamp = (x, y);
    }
}

fn draw_breakout_game(frame: &mut ratatui::Frame, game: &BreakoutGame) {
    let area = frame.area();

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec score et vies
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(4), // Footer avec instructions
    ])
    .split(area);

    // Fond sombre
    let background = Block::new().style(Style::default().bg(Color::Black));
    frame.render_widget(background, area);

    // === HEADER ===
    let lives_hearts = "♥ ".repeat(game.lives as usize);
    let header_text = vec![
        Line::from(vec![
            "🧱 ".yellow().bold(),
            "BREAKOUT".cyan().bold(),
            " 🧱".yellow().bold(),
        ]),
        Line::from(vec![
            "Score: ".white(),
            format!("{}", game.score).yellow().bold(),
            "  Lives: ".white(),
            format!("{}", game.lives).red().bold(),
            " ".white(),
            lives_hearts.red().bold(),
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

    // === ZONE DE JEU ===
    let game_area = chunks[1];
    let game_block = Block::bordered()
        .title(" Game Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(5, 10, 15)));
    frame.render_widget(game_block, game_area);

    let inner_area = game_area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 2,
    });

    // Calculer l'offset pour centrer le terrain
    let field_start_x = inner_area.x + (inner_area.width.saturating_sub(FIELD_WIDTH)) / 2;
    let field_start_y = inner_area.y + (inner_area.height.saturating_sub(FIELD_HEIGHT)) / 2;
    let (sx, sy) = game.shake.offset();
    let ox = field_start_x as i32 + sx;
    let oy = field_start_y as i32 + sy;
    let buf = frame.buffer_mut();

    // Briques (une brique détruite reste un instant en flash blanc)
    for (r, row) in game.bricks.iter().enumerate() {
        for (c, brick) in row.iter().enumerate() {
            let f = game.brick_flash[r][c];
            let col = match (brick.destroyed, f > 0.0) {
                (false, _) => brick.color,
                (true, true) => fx::lerp(brick.color, (255, 255, 255), f),
                (true, false) => continue,
            };
            let (bx, by) = (ox + brick.x as i32, oy + brick.y as i32);
            if !brick.destroyed {
                fx::glow(buf, bx + BRICK_WIDTH as i32 / 2, by, 2, brick.color, 0.12);
            }
            for dx in 0..BRICK_WIDTH as i32 {
                fx::put(buf, bx + dx, by, '█', col);
            }
        }
    }

    // Raquette: grosse lueur, flash au rebond, ombre portée sous la barre
    let (pw, py) = (PADDLE_WIDTH as i32, game.paddle.y as i32);
    let px = game.paddle.x as i32;
    let pf = game.paddle_flash;
    let glow_c = fx::lerp((90, 140, 255), (255, 255, 255), pf);
    fx::glow(buf, ox + px + pw / 2, oy + py, 5, glow_c, 0.6 + 0.35 * pf);
    for dx in 0..pw {
        let t = (dx as f32 / (pw - 1) as f32 - 0.5).abs() * 2.0; // bords plus sombres
        let base = fx::lerp((255, 255, 255), (90, 140, 255), t * t);
        fx::put(
            buf,
            ox + px + dx,
            oy + py,
            '█',
            fx::lerp(base, (255, 255, 255), pf),
        );
        fx::put(
            buf,
            ox + px + dx,
            oy + py + 1,
            '▀',
            fx::lerp((20, 30, 70), glow_c, 0.4 + 0.5 * pf),
        );
    }

    // Balle (Braille) + traînée
    game.trail.draw(buf, ox, oy, (255, 235, 130), (90, 60, 20));
    game.particles.draw(buf, (ox, oy));

    if game.state == GameState::GameOver {
        fx::fade_to_black(buf, inner_area, game.fade * 0.7);
    }

    // === FOOTER ===
    let instructions = match game.state {
        GameState::Playing => {
            if game.ball_stuck {
                vec![
                    Line::from(vec![
                        "←→".cyan().bold(),
                        " Move  ".white(),
                        "SPACE".green().bold(),
                        " Launch  ".white(),
                        "P".yellow().bold(),
                        " Pause  ".white(),
                        "R".green().bold(),
                        " Restart  ".white(),
                        "Q".red().bold(),
                        " Quit".white(),
                    ]),
                    Line::from(vec![
                        "M".yellow().bold(),
                        " Music  ".white(),
                        "N".yellow().bold(),
                        " Sound Effects".white(),
                    ]),
                ]
            } else {
                vec![
                    Line::from(vec![
                        "←→".cyan().bold(),
                        " Move Paddle  ".white(),
                        "P".yellow().bold(),
                        " Pause  ".white(),
                        "R".green().bold(),
                        " Restart  ".white(),
                        "Q".red().bold(),
                        " Quit".white(),
                    ]),
                    Line::from(vec![
                        "M".yellow().bold(),
                        " Music  ".white(),
                        "N".yellow().bold(),
                        " Sound Effects".white(),
                    ]),
                ]
            }
        }
        GameState::Paused => vec![
            Line::from(vec![
                "PAUSED".yellow().bold(),
                "  ".white(),
                "P".green().bold(),
                " Resume  ".white(),
                "R".green().bold(),
                " Restart  ".white(),
                "Q".red().bold(),
                " Quit".white(),
            ]),
            Line::from(vec![
                "M".yellow().bold(),
                " Music  ".white(),
                "N".yellow().bold(),
                " Sound Effects".white(),
            ]),
        ],
        GameState::GameOver | GameState::Victory => vec![
            Line::from(vec![
                if game.state == GameState::Victory {
                    "🎉 VICTORY! 🎉".green().bold()
                } else {
                    "💥 GAME OVER 💥".red().bold()
                },
                "  ".white(),
                "R".green().bold(),
                " Restart  ".white(),
                "Q".red().bold(),
                " Quit".white(),
            ]),
            Line::from(vec![
                "M".yellow().bold(),
                " Music  ".white(),
                "N".yellow().bold(),
                " Sound Effects".white(),
            ]),
        ],
    };

    let footer = Paragraph::new(instructions)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(footer, chunks[2]);

    // === POPUPS ===
    if game.state == GameState::GameOver {
        let popup_width = 40.min(area.width);
        let popup_height = 8.min(area.height);
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

        let game_over_text = vec![
            Line::from(""),
            Line::from("💥 GAME OVER 💥".red().bold()),
            Line::from(""),
            Line::from(vec![
                "Final Score: ".white(),
                format!("{}", game.score).yellow().bold(),
            ]),
            Line::from(""),
            Line::from(vec![
                "Press ".gray(),
                "R".green().bold(),
                " to restart or ".gray(),
                "Q".red().bold(),
                " to quit".gray(),
            ]),
        ];

        let popup = Paragraph::new(game_over_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Game Over ".red().bold())
                    .border_style(Style::new().red().bold())
                    .style(Style::default().bg(Color::Black)),
            );

        frame.render_widget(popup, popup_area);
    } else if game.state == GameState::Victory {
        let popup_width = 40.min(area.width);
        let popup_height = 8.min(area.height);
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

        let victory_text = vec![
            Line::from(""),
            Line::from("🎉 VICTORY! 🎉".green().bold()),
            Line::from(""),
            Line::from(vec![
                "Final Score: ".white(),
                format!("{}", game.score).yellow().bold(),
            ]),
            Line::from(""),
            Line::from(vec![
                "Press ".gray(),
                "R".green().bold(),
                " to restart or ".gray(),
                "Q".red().bold(),
                " to quit".gray(),
            ]),
        ];

        let popup = Paragraph::new(victory_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Victory! ".green().bold())
                    .border_style(Style::new().green().bold())
                    .style(Style::default().bg(Color::Rgb(0, 50, 0))),
            );

        frame.render_widget(popup, popup_area);
    }
}
