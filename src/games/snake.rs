use crate::audio::{AudioManager, SoundEffect};
use crate::core::{Game, GameAction};
use crate::engine::{
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
    x: u16,
    y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnakeDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct SnakeGame {
    snake: Vec<Position>,
    direction: SnakeDirection,
    food: Position,
    score: u32,
    game_over: bool,
    width: u16,
    height: u16,
    audio: AudioManager,
    music_started: bool,
    highscore_manager: HighScoreManager,
    start_time: std::time::Instant,
    score_saved: bool,
    particles: Particles,
    shake: Shake,
    time: f32,
    fade: f32,
    origin: (u16, u16), // coin du terrain (cellules terminal), mis à jour au draw
}

impl SnakeGame {
    pub fn new() -> Self {
        // Dimensions par défaut, seront mises à jour lors du premier rendu
        let width = 40;
        let height = 20;
        let snake = vec![Position {
            x: width / 2,
            y: height / 2,
        }];
        let food = Self::generate_food(&snake, width, height);

        Self {
            snake,
            direction: SnakeDirection::Right,
            food,
            score: 0,
            game_over: false,
            width,
            height,
            audio: AudioManager::default(),
            music_started: false,
            highscore_manager: HighScoreManager::default(),
            start_time: std::time::Instant::now(),
            score_saved: false,
            particles: Particles::new(256, 12.0),
            shake: Shake::default(),
            time: 0.0,
            fade: 0.0,
            origin: (0, 0),
        }
    }

    /// Centre terminal d'une cellule logique (2 caractères de large).
    fn cell_center(&self, p: Position) -> (f32, f32) {
        (
            (self.origin.0 + p.x * 2) as f32 + 1.0,
            (self.origin.1 + p.y) as f32 + 0.5,
        )
    }

    fn generate_food(snake: &[Position], width: u16, height: u16) -> Position {
        let mut rng = crate::engine::rng::game_rng();
        loop {
            let food = Position {
                x: rng.random_range(0..width),
                y: rng.random_range(0..height),
            };
            if !snake.contains(&food) {
                return food;
            }
        }
    }

    fn move_snake(&mut self) {
        if self.game_over {
            return;
        }

        let head = self.snake[0];
        let new_head = match self.direction {
            SnakeDirection::Up => Position {
                x: head.x,
                y: head.y.saturating_sub(1),
            },
            SnakeDirection::Down => Position {
                x: head.x,
                y: head.y + 1,
            },
            SnakeDirection::Left => Position {
                x: head.x.saturating_sub(1),
                y: head.y,
            },
            SnakeDirection::Right => Position {
                x: head.x + 1,
                y: head.y,
            },
        };

        if new_head.x >= self.width || new_head.y >= self.height || self.snake.contains(&new_head) {
            self.game_over = true;
            // Arrêter la musique et jouer le son de game over
            self.audio.stop_music();
            self.audio.play_sound(SoundEffect::SnakeGameOver);
            let (x, y) = self.cell_center(head);
            self.particles
                .burst(x, y, 40, 14.0, (255, 120, 60), (60, 0, 0));
            self.shake.kick(2.0);

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
            self.music_started = false;
            return;
        }

        self.snake.insert(0, new_head);

        if new_head == self.food {
            self.score += 10;
            self.audio.play_sound(SoundEffect::SnakeEat);
            let (x, y) = self.cell_center(new_head);
            self.particles
                .burst(x, y, 16, 9.0, (255, 230, 120), (200, 30, 30));
            self.food = Self::generate_food(&self.snake, self.width, self.height);
        } else {
            self.snake.pop();
        }
    }

    // Méthode pour mettre à jour les dimensions du jeu
    pub fn update_dimensions(&mut self, new_width: u16, new_height: u16) {
        if self.width != new_width || self.height != new_height {
            self.width = new_width;
            self.height = new_height;

            // Assurer que le serpent reste dans les limites
            for segment in &mut self.snake {
                if segment.x >= new_width {
                    segment.x = new_width - 1;
                }
                if segment.y >= new_height {
                    segment.y = new_height - 1;
                }
            }

            // Repositionner la nourriture si nécessaire
            if self.food.x >= new_width || self.food.y >= new_height {
                self.food = Self::generate_food(&self.snake, new_width, new_height);
            }
        }
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() {
            // Choisir la version de la musique selon la longueur du serpent
            if self.snake.len() >= 15 {
                self.audio.play_snake_music_fast(); // Version rapide pour serpent long
            } else {
                self.audio.play_snake_music(); // Version normale
            }
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started && self.audio.is_music_enabled() && self.audio.is_music_empty() {
            // Choisir la version appropriée selon la longueur actuelle
            if self.snake.len() >= 15 {
                self.audio.play_snake_music_fast();
            } else {
                self.audio.play_snake_music();
            }
        }
    }

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // Vérifier si c'est un high score
        if self.highscore_manager.is_high_score("snake", self.score) {
            let duration = self.start_time.elapsed().as_secs();
            let game_data = GameData::Snake {
                length: self.snake.len(),
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), self.score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("snake", score) {
                self.score_saved = true;
            }
        }
    }
}

impl Game for SnakeGame {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        if self.game_over {
            match key.code {
                KeyCode::Char('r') => {
                    // Nettoyer l'audio avant de redémarrer
                    self.audio.clear_effects();
                    self.audio.stop_music();
                    *self = Self::new();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            }
        } else {
            match key.code {
                KeyCode::Up if self.direction != SnakeDirection::Down => {
                    self.direction = SnakeDirection::Up;
                    GameAction::Continue
                }
                KeyCode::Down if self.direction != SnakeDirection::Up => {
                    self.direction = SnakeDirection::Down;
                    GameAction::Continue
                }
                KeyCode::Left if self.direction != SnakeDirection::Right => {
                    self.direction = SnakeDirection::Left;
                    GameAction::Continue
                }
                KeyCode::Right if self.direction != SnakeDirection::Left => {
                    self.direction = SnakeDirection::Right;
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                // Touches pour contrôler l'audio (optionnel)
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
    }

    fn update(&mut self) -> GameAction {
        if !self.game_over {
            // Démarrer la musique si ce n'est pas encore fait
            self.start_music_if_needed();

            self.move_snake();
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_snake_game(frame, self);
    }

    fn frame_time(&self) -> Option<Duration> {
        Some(Duration::from_millis(16))
    }

    fn animate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.time += dt;
        self.particles.update(dt);
        self.shake.update(dt);
        if self.game_over {
            self.fade = (self.fade + dt / 0.8).min(1.0);
        }
    }

    fn tick_rate(&self) -> Duration {
        // Vitesse de base: 300ms
        let base_speed: u64 = 300;

        // Réduction de 15ms par segment du serpent (sans compter la tête)
        let speed_increase = (self.snake.len().saturating_sub(1) * 15) as u64;

        // Vitesse minimale: 80ms pour éviter que ce soit injouable
        let final_speed = base_speed.saturating_sub(speed_increase).max(80);

        Duration::from_millis(final_speed)
    }
}

fn draw_snake_game(frame: &mut ratatui::Frame, app: &mut SnakeGame) {
    let area = frame.area();

    // Layout principal d'abord pour connaître l'espace réel disponible
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec score
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer avec instructions
    ])
    .split(area);

    let game_area = chunks[1];
    let inner_area = game_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    // Calculer les dimensions en cellules de 2 caractères de large (comme Tetris)
    let game_width = (inner_area.width / 2).max(10); // Division par 2 pour des cellules de 2 chars
    let game_height = inner_area.height.max(10);

    // Mettre à jour les dimensions logiques du jeu
    app.update_dimensions(game_width, game_height);

    // Fond sombre élégant
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let current_speed = app.tick_rate().as_millis();
    let snake_length = app.snake.len();
    let audio_status = if app.audio.is_enabled() {
        "🔊"
    } else {
        "🔇"
    };

    let header_text = vec![
        Line::from(vec![
            "🐍 ".green().bold(),
            "SNAKE GAME".cyan().bold(),
            " 🐍".green().bold(),
        ]),
        Line::from(vec![
            "Score: ".yellow(),
            format!("{}", app.score).white().bold(),
            " | Length: ".gray(),
            format!("{snake_length}").green().bold(),
            " | Speed: ".gray(),
            format!("{current_speed}ms").red().bold(),
            " | Audio: ".gray(),
            audio_status.white(),
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
        .title(" Playing Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    frame.render_widget(game_block, game_area);

    let inner_area = game_area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    app.origin = (inner_area.x, inner_area.y);
    let (sx, sy) = app.shake.offset();
    let ox = inner_area.x as i32 + sx;
    let oy = inner_area.y as i32 + sy;
    let buf = frame.buffer_mut();

    // Grille de fond subtile
    for y in 0..game_height as i32 {
        for x in 0..game_width as i32 {
            fx::put(buf, ox + x * 2, oy + y, '░', (30, 35, 40));
            fx::put(buf, ox + x * 2 + 1, oy + y, '░', (30, 35, 40));
        }
    }

    // Lueurs (fond), avant les entités
    let pulse = 0.35 + 0.15 * (app.time * 6.0).sin();
    let food = app.food;
    if food.x < game_width && food.y < game_height {
        fx::glow(
            buf,
            ox + food.x as i32 * 2 + 1,
            oy + food.y as i32,
            4,
            (255, 60, 60),
            pulse,
        );
    }
    if !app.game_over {
        let h = app.snake[0];
        fx::glow(
            buf,
            ox + h.x as i32 * 2 + 1,
            oy + h.y as i32,
            5,
            (80, 255, 120),
            0.55,
        );
    }

    // Nourriture
    if food.x < game_width && food.y < game_height {
        let c = fx::lerp((255, 60, 60), (255, 200, 120), pulse);
        for dx in 0..2 {
            fx::put(buf, ox + food.x as i32 * 2 + dx, oy + food.y as i32, '█', c);
        }
    }

    // Serpent: dégradé lisse tête → queue
    let len = app.snake.len().max(2) as f32 - 1.0;
    for (i, seg) in app.snake.iter().enumerate() {
        if seg.x < game_width && seg.y < game_height {
            let c = fx::lerp((140, 255, 140), (20, 90, 70), i as f32 / len);
            for dx in 0..2 {
                fx::put(buf, ox + seg.x as i32 * 2 + dx, oy + seg.y as i32, '█', c);
            }
        }
    }

    app.particles.draw(buf, (sx, sy));

    // Fondu du terrain au game over
    if app.game_over {
        fx::fade_to_black(buf, inner_area, app.fade * 0.7);
    }

    // === FOOTER ===
    let instructions = vec![Line::from(vec![
        "Arrow Keys".cyan().bold(),
        " Move  ".white(),
        "M".yellow().bold(),
        " Music  ".white(),
        "N".blue().bold(),
        " Audio  ".white(),
        "Q".red().bold(),
        " Quit  ".white(),
        if app.game_over {
            "R".green().bold()
        } else {
            "".white()
        },
        if app.game_over { " Restart" } else { "" }.white(),
    ])];

    let footer = Paragraph::new(instructions)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(footer, chunks[2]);

    // === GAME OVER POPUP ===
    if app.game_over {
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

        // Fond transparent
        frame.render_widget(Clear, popup_area);

        let game_over_text = vec![
            Line::from(""),
            Line::from("💀 GAME OVER 💀".red().bold()),
            Line::from(""),
            Line::from(vec![
                "Final Score: ".white(),
                format!("{}", app.score).yellow().bold(),
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
    }
}
