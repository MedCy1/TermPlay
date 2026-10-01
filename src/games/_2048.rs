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

// Taille de la grille 2048
const GRID_SIZE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Game2048 {
    grid: [[u32; GRID_SIZE]; GRID_SIZE],
    score: u32,
    best_score: u32,
    game_over: bool,
    won: bool,
    moved: bool, // Pour savoir si le dernier mouvement a changé quelque chose

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
    flash: [[f32; GRID_SIZE]; GRID_SIZE],
    merges: Vec<(usize, usize, u32)>, // (ligne, colonne, valeur) du dernier coup
    slides: Vec<Slide>,
    sliding: bool,
    slide_t: f32,
    spawn: Option<(usize, usize)>, // nouvelle tuile, cachée pendant le glissement
    pop: [[f32; GRID_SIZE]; GRID_SIZE],
    floats: Vec<Float>,
    fade: f32,
    origin: (u16, u16), // coin de la grille, mis à jour au draw
}

const SLIDE_TIME: f32 = 0.09;
const POP_TIME: f32 = 0.12;

#[derive(Clone, Copy)]
struct Slide {
    from: (usize, usize),
    to: (usize, usize),
    value: u32,
}

/// Texte flottant "+N" au-dessus d'une fusion.
struct Float {
    x: f32,
    y: f32,
    value: u32,
    life: f32,
}

const CELL_W: u16 = 8;
const CELL_H: u16 = 3;

impl Game2048 {
    pub fn new() -> Self {
        let highscore_manager = HighScoreManager::default();

        // Charger le meilleur score depuis le fichier de high scores
        let best_score = highscore_manager
            .get_best_score("2048")
            .map(|score| score.score)
            .unwrap_or(0);

        let mut game = Self {
            grid: [[0; GRID_SIZE]; GRID_SIZE],
            score: 0,
            best_score,
            game_over: false,
            won: false,
            moved: false,

            audio: AudioManager::default(),
            music_started: false,

            highscore_manager,
            start_time: std::time::Instant::now(),
            score_saved: false,
            particles: Particles::new(384, 10.0),
            shake: Shake::default(),
            flash: [[0.0; GRID_SIZE]; GRID_SIZE],
            merges: Vec::with_capacity(8),
            slides: Vec::with_capacity(16),
            sliding: false,
            slide_t: 0.0,
            spawn: None,
            pop: [[0.0; GRID_SIZE]; GRID_SIZE],
            floats: Vec::with_capacity(8),
            fade: 0.0,
            origin: (0, 0),
        };

        // Ajouter deux tuiles au début
        game.add_random_tile();
        game.add_random_tile();

        game
    }

    fn add_random_tile(&mut self) {
        let empty_cells: Vec<(usize, usize)> = (0..GRID_SIZE)
            .flat_map(|row| (0..GRID_SIZE).map(move |col| (row, col)))
            .filter(|&(r, c)| self.grid[r][c] == 0)
            .collect();

        if empty_cells.is_empty() {
            return;
        }

        let mut rng = crate::engine::rng::game_rng();
        let &(row, col) = empty_cells.choose(&mut rng).unwrap();

        // 90% chance pour 2, 10% chance pour 4
        let value = if rng.random_bool(0.9) { 2 } else { 4 };
        self.grid[row][col] = value;
        self.spawn = Some((row, col));
    }

    fn can_move(&self) -> bool {
        // Vérifier s'il y a des cellules vides
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                if self.grid[row][col] == 0 {
                    return true;
                }
            }
        }

        // Vérifier s'il y a des fusions possibles
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let current = self.grid[row][col];

                // Vérifier à droite
                if col < GRID_SIZE - 1 && self.grid[row][col + 1] == current {
                    return true;
                }

                // Vérifier en bas
                if row < GRID_SIZE - 1 && self.grid[row + 1][col] == current {
                    return true;
                }
            }
        }

        false
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() && !self.game_over {
            // Choisir la version selon le score actuel
            if self.score >= 10000 {
                self.audio.play_2048_music_fast(); // Version énergique pour scores élevés
            } else {
                self.audio.play_2048_music(); // Version zen normale
            }
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started
            && self.audio.is_music_enabled()
            && !self.game_over
            && self.audio.is_music_empty()
        {
            // Choisir la version appropriée selon le score actuel
            if self.score >= 10000 {
                self.audio.play_2048_music_fast();
            } else {
                self.audio.play_2048_music();
            }
        }
    }

    /// Termine le glissement en cours: déclenche les effets de fusion.
    fn finish_slide(&mut self) {
        self.sliding = false;
        self.spawn = None;
        for i in 0..self.merges.len() {
            let (r, c, v) = self.merges[i];
            let rgb = Self::tile_rgb(v);
            let dark = fx::lerp(rgb, (0, 0, 0), 0.8);
            let x0 = (self.origin.0 + c as u16 * (CELL_W + 1)) as f32;
            let y0 = (self.origin.1 + r as u16 * (CELL_H + 1)) as f32;
            let (x1, y1) = (x0 + CELL_W as f32 - 1.0, y0 + CELL_H as f32 - 1.0);
            for (x, y) in [(x0, y0), (x1, y0), (x0, y1), (x1, y1)] {
                self.particles.burst(x, y, 6, 11.0, (255, 255, 255), rgb);
                self.particles.burst(x, y, 4, 8.0, rgb, dark);
            }
            self.particles
                .burst(x0 + 3.5, y0 + 1.0, 8, 6.0, (255, 255, 255), dark);
            self.flash[r][c] = 1.0;
            self.pop[r][c] = 1.0;
            if v >= 128 && fx::floating_scores() {
                self.floats.push(Float {
                    x: x0 + 2.0,
                    y: y0 - 1.0,
                    value: v,
                    life: 0.8,
                });
            }
            match v {
                256 | 512 => self.shake.kick(0.3),
                1024.. => self.shake.kick(1.0),
                _ => {}
            }
        }
        self.merges.clear();
    }

    /// Applique un coup: logique pure dans `compute_move`, puis audio / victoire.
    fn move_tiles(&mut self, direction: Direction) {
        if self.sliding {
            self.finish_slide();
        }
        self.moved = false;
        let (new_grid, gained) =
            compute_move(&self.grid, direction, &mut self.slides, &mut self.merges);
        self.score += gained;
        for _ in 0..self.merges.len() {
            self.audio.play_sound(SoundEffect::Game2048Merge);
        }
        if !self.won && self.merges.iter().any(|m| m.2 == 2048) {
            self.won = true;
            self.audio.play_sound(SoundEffect::Game2048Victory);
            self.audio.stop_music();
            self.audio.play_2048_music_celebration();
            self.music_started = false;
            self.save_high_score_if_needed();
        }

        self.moved = new_grid != self.grid;
        self.grid = new_grid;
        if self.moved {
            self.sliding = true;
            self.slide_t = 0.0;
        } else {
            self.slides.clear();
            self.merges.clear();
        }

        // Ajouter une nouvelle tuile si quelque chose a bougé
        if self.moved {
            self.add_random_tile();

            // Vérifier la fin de jeu
            if !self.can_move() {
                self.game_over = true;
                self.audio.play_sound(SoundEffect::Game2048GameOver);

                // Sauvegarder le score si c'est un high score et pas encore sauvé
                self.save_high_score_if_needed();
            }
        }

        // Mettre à jour le meilleur score
        if self.score > self.best_score {
            self.best_score = self.score;
        }
    }

    fn restart(&mut self) {
        self.grid = [[0; GRID_SIZE]; GRID_SIZE];
        self.score = 0;
        self.game_over = false;
        self.won = false;
        self.moved = false;
        self.score_saved = false;
        self.fade = 0.0;
        self.sliding = false;
        self.slides.clear();
        self.merges.clear();
        self.floats.clear();
        self.pop = [[0.0; GRID_SIZE]; GRID_SIZE];
        self.flash = [[0.0; GRID_SIZE]; GRID_SIZE];
        self.start_time = std::time::Instant::now();

        self.add_random_tile();
        self.add_random_tile();
    }

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // Vérifier si c'est un high score
        if self.highscore_manager.is_high_score("2048", self.score) {
            let duration = self.start_time.elapsed().as_secs();

            // Trouver la plus haute tuile atteinte
            let highest_tile = self
                .grid
                .iter()
                .flat_map(|row| row.iter())
                .cloned()
                .max()
                .unwrap_or(0);

            // Estimer le nombre de mouvements basé sur le score et le niveau
            let estimated_moves = (self.score / 10).max(10); // Estimation basée sur le score

            let game_data = GameData::Game2048 {
                highest_tile,
                moves: estimated_moves,
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), self.score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("2048", score) {
                self.score_saved = true;
            }
        }
    }

    /// Progression: tuiles discrètes (2-4) puis ambre → corail → magenta → violet → or/néon.
    fn tile_rgb(value: u32) -> fx::Rgb {
        match value {
            0 => (30, 35, 48),
            2 => (78, 88, 112),
            4 => (100, 98, 140),
            8 => (255, 170, 50),
            16 => (255, 125, 40),
            32 => (255, 90, 80),
            64 => (255, 50, 100),
            128 => (235, 50, 170),
            256 => (190, 50, 235),
            512 => (125, 70, 255),
            1024 => (255, 200, 40),
            2048 => (255, 250, 150),
            _ => (120, 255, 220),
        }
    }
}

impl Game for Game2048 {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        if self.game_over || self.won {
            match key.code {
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
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') => {
                    self.move_tiles(Direction::Up);
                    if self.moved {
                        self.audio.play_sound(SoundEffect::Game2048Move);
                    }
                    GameAction::Continue
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.move_tiles(Direction::Down);
                    if self.moved {
                        self.audio.play_sound(SoundEffect::Game2048Move);
                    }
                    GameAction::Continue
                }
                KeyCode::Left | KeyCode::Char('a') => {
                    self.move_tiles(Direction::Left);
                    if self.moved {
                        self.audio.play_sound(SoundEffect::Game2048Move);
                    }
                    GameAction::Continue
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.move_tiles(Direction::Right);
                    if self.moved {
                        self.audio.play_sound(SoundEffect::Game2048Move);
                    }
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
            }
        }
    }

    fn frame_time(&self) -> Option<Duration> {
        Some(Duration::from_millis(16))
    }

    fn animate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.particles.update(dt);
        self.shake.update(dt);
        for f in self.flash.iter_mut().flatten() {
            *f = (*f - dt / 0.25).max(0.0);
        }
        for p in self.pop.iter_mut().flatten() {
            *p = (*p - dt / POP_TIME).max(0.0);
        }
        self.floats.retain_mut(|f| {
            f.life -= dt;
            f.y -= 5.0 * dt;
            f.life > 0.0
        });
        if self.sliding {
            self.slide_t += dt;
            if self.slide_t >= SLIDE_TIME {
                self.finish_slide();
            }
        }
        if self.game_over {
            self.fade = (self.fade + dt / 0.8).min(1.0);
        }
    }

    fn update(&mut self) -> GameAction {
        self.start_music_if_needed();
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_2048_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(100) // Pas besoin d'être très rapide pour 2048
    }
}

fn draw_2048_game(frame: &mut ratatui::Frame, game: &mut Game2048) {
    let area = frame.area();

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec score
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(4), // Footer avec instructions
    ])
    .split(area);

    // Fond sombre élégant
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let header_text = vec![
        Line::from(vec![
            "🎮 ".yellow().bold(),
            "2048 GAME".cyan().bold(),
            " 🎮".yellow().bold(),
        ]),
        Line::from(vec![
            "Score: ".yellow(),
            format!("{}", game.score).white().bold(),
            " | Best: ".gray(),
            format!("{}", game.best_score).green().bold(),
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
        horizontal: 2,
    });

    // Calculer les dimensions pour centrer la grille
    let cell_width = CELL_W;
    let cell_height = CELL_H;
    let grid_width = (GRID_SIZE as u16 * cell_width) + (GRID_SIZE as u16 - 1); // +espaces entre cellules
    let grid_height = (GRID_SIZE as u16 * cell_height) + (GRID_SIZE as u16 - 1);

    let start_x = inner_area.x + (inner_area.width.saturating_sub(grid_width)) / 2;
    let start_y = inner_area.y + (inner_area.height.saturating_sub(grid_height)) / 2;
    game.origin = (start_x, start_y);
    let (sx, sy) = game.shake.offset();
    let (max_x, max_y) = (
        area.width.saturating_sub(grid_width) as i32,
        area.height.saturating_sub(grid_height) as i32,
    );
    let start_x = (start_x as i32 + sx).clamp(0, max_x) as u16;
    let start_y = (start_y as i32 + sy).clamp(0, max_y) as u16;

    let buf = frame.buffer_mut();
    let (cw, ch) = (cell_width as i32, cell_height as i32);
    let pitch = |n: usize, size: i32, origin: u16| origin as i32 + n as i32 * (size + 1);

    // Cases vides + lueur des grosses tuiles / fusions
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let (x, y) = (pitch(col, cw, start_x), pitch(row, ch, start_y));
            draw_tile(buf, x, y, cw, ch, 0, 0.0);
            let v = if game.sliding { 0 } else { game.grid[row][col] };
            let g = game.pop[row][col].max(if v >= 512 { 0.3 } else { 0.0 });
            if g > 0.0 {
                fx::glow(
                    buf,
                    x + cw / 2,
                    y + ch / 2,
                    4,
                    Game2048::tile_rgb(v.max(8)),
                    g * 0.6,
                );
            }
        }
    }

    if game.sliding {
        // Glissement: positions interpolées (ease-out), valeurs d'avant fusion
        let t = (game.slide_t / SLIDE_TIME).min(1.0);
        let e = 1.0 - (1.0 - t) * (1.0 - t);
        for sl in game.slides.iter() {
            let lerp = |a: usize, b: usize, size: i32, origin: u16| {
                let (pa, pb) = (pitch(a, size, origin) as f32, pitch(b, size, origin) as f32);
                (pa + (pb - pa) * e).round() as i32
            };
            let x = lerp(sl.from.1, sl.to.1, cw, start_x);
            let y = lerp(sl.from.0, sl.to.0, ch, start_y);
            draw_tile(buf, x, y, cw, ch, sl.value, 0.0);
        }
    } else {
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let v = game.grid[row][col];
                if v == 0 {
                    continue;
                }
                let (mut x, mut y, mut w, mut h) =
                    (pitch(col, cw, start_x), pitch(row, ch, start_y), cw, ch);
                let pop = game.pop[row][col];
                if pop > 0.4 {
                    // zoom: la case résultante déborde d'une cellule
                    (x, y, w, h) = (x - 1, y - 1, w + 2, h + 2);
                }
                draw_tile(buf, x, y, w, h, v, game.flash[row][col].max(pop));
            }
        }
    }

    for f in game.floats.iter() {
        let c = fx::lerp(Game2048::tile_rgb(f.value), (255, 255, 255), 0.5);
        let mut n = f.value;
        let mut digits = [b'0'; 10];
        let mut i = digits.len();
        while n > 0 {
            i -= 1;
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        fx::put(buf, f.x as i32, f.y as i32, '+', c);
        for (k, d) in digits[i..].iter().enumerate() {
            fx::put(buf, f.x as i32 + 1 + k as i32, f.y as i32, *d as char, c);
        }
    }

    let grid_rect = Rect::new(start_x, start_y, grid_width, grid_height);
    if game.game_over {
        fx::fade_to_black(frame.buffer_mut(), grid_rect, game.fade * 0.7);
    }
    game.particles.draw(frame.buffer_mut(), (0, 0));

    // === FOOTER ===
    let instructions = if game.game_over || game.won {
        vec![
            Line::from(vec![
                if game.won {
                    "🎉 YOU WON! 🎉".green().bold()
                } else {
                    "GAME OVER".red().bold()
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
        ]
    } else {
        vec![
            Line::from(vec![
                "↑↓←→".cyan().bold(),
                " or ".white(),
                "WASD".cyan().bold(),
                " Move  ".white(),
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

    // === GAME OVER POPUP ===
    if game.game_over {
        let popup_width = 50.min(area.width);
        let popup_height = 10.min(area.height);
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
            Line::from("💀 GAME OVER 💀".red().bold()),
            Line::from(""),
            Line::from(vec![
                "Final Score: ".white(),
                format!("{}", game.score).yellow().bold(),
            ]),
            Line::from(vec![
                "Best Score: ".white(),
                format!("{}", game.best_score).green().bold(),
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
    // === POPUP DE VICTOIRE ===
    else if game.won {
        let popup_width = 50.min(area.width);
        let popup_height = 10.min(area.height);
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Fond semi-transparent
        frame.render_widget(Clear, popup_area);

        let win_text = vec![
            Line::from(""),
            Line::from("🎉 CONGRATULATIONS! 🎉".green().bold()),
            Line::from(""),
            Line::from("You reached 2048!".white()),
            Line::from(""),
            Line::from(vec![
                "Continue playing or ".white(),
                "R".green().bold(),
                "estart".white(),
            ]),
        ];

        let win_popup = Paragraph::new(win_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::bordered()
                    .title(" Victory! ".green().bold())
                    .border_style(Style::new().green())
                    .style(Style::default().bg(Color::Rgb(0, 50, 0))),
            );

        frame.render_widget(win_popup, popup_area);
    }
}

// Trait extension pour Vec::choose (simulation)
trait Choose<T> {
    fn choose<R: rand::Rng>(&self, rng: &mut R) -> Option<&T>;
}

impl<T> Choose<T> for Vec<T> {
    fn choose<R: rand::Rng>(&self, rng: &mut R) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            let index = rng.random_range(0..self.len());
            self.get(index)
        }
    }
}

/// Tuile en relief: bordure arrondie, dégradé vertical (clair en haut), ombre portée dessous.
fn draw_tile(
    buf: &mut ratatui::buffer::Buffer,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    value: u32,
    flash: f32,
) {
    let base = Game2048::tile_rgb(value);
    let white = (255, 255, 255);
    let (border, top, bottom) = if value == 0 {
        ((48, 56, 74), base, base)
    } else {
        (
            fx::lerp(base, white, 0.3 + 0.5 * flash),
            fx::lerp(base, white, 0.12 + 0.6 * flash),
            fx::lerp(base, (0, 0, 0), 0.3),
        )
    };
    if value != 0 {
        for dx in 1..w {
            fx::put(buf, x + dx, y + h, '▀', fx::lerp(base, (0, 0, 0), 0.85));
        }
    }
    let text_c = match value {
        0 => base,
        1024.. => (30, 20, 0),
        2 | 4 => (225, 230, 245),
        _ => (255, 255, 255),
    };
    let mut digits = [b' '; 10];
    let mut i = digits.len();
    let mut n = value;
    while n > 0 {
        i -= 1;
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    let len = (digits.len() - i) as i32;
    for dy in 0..h {
        let bg = fx::lerp(top, bottom, dy as f32 / (h - 1).max(1) as f32);
        for dx in 0..w {
            let edge_x = dx == 0 || dx == w - 1;
            let ch = match (dy == 0, dy == h - 1, edge_x) {
                (true, _, true) => {
                    if dx == 0 {
                        '╭'
                    } else {
                        '╮'
                    }
                }
                (_, true, true) => {
                    if dx == 0 {
                        '╰'
                    } else {
                        '╯'
                    }
                }
                (true, _, _) | (_, true, _) => '─',
                (_, _, true) => '│',
                _ => ' ',
            };
            let is_border = ch != ' ';
            let (mut c, mut b) = (if is_border { border } else { text_c }, bg);
            let tx = dx - (w - len) / 2;
            let glyph = if value != 0 && dy == h / 2 && (0..len).contains(&tx) {
                c = text_c;
                digits[i + tx as usize] as char
            } else {
                ch
            };
            if value == 0 && !is_border {
                b = base;
            }
            fx::put_bg(buf, x + dx, y + dy, glyph, c, b, glyph.is_ascii_digit());
        }
    }
}

type Grid = [[u32; GRID_SIZE]; GRID_SIZE];

/// Une ligne à la fois: tasse les tuiles vers le bord du coup et fusionne les paires
/// (une tuile ne fusionne qu'une fois). Retourne la grille et le score gagné;
/// remplit `slides` (déplacements) et `merges` (cases fusionnées).
fn compute_move(
    grid: &Grid,
    direction: Direction,
    slides: &mut Vec<Slide>,
    merges: &mut Vec<(usize, usize, u32)>,
) -> (Grid, u32) {
    slides.clear();
    merges.clear();
    let mut new_grid = [[0u32; GRID_SIZE]; GRID_SIZE];
    let mut gained = 0;
    let last = GRID_SIZE - 1;

    for i in 0..GRID_SIZE {
        // k = rang dans le sens du déplacement
        let pos = |k: usize| match direction {
            Direction::Left => (i, k),
            Direction::Right => (i, last - k),
            Direction::Up => (k, i),
            Direction::Down => (last - k, i),
        };
        let mut out = [0u32; GRID_SIZE];
        let mut merged = [false; GRID_SIZE];
        let mut t = 0;
        for k in 0..GRID_SIZE {
            let (r, c) = pos(k);
            let v = grid[r][c];
            if v == 0 {
                continue;
            }
            if t > 0 && out[t - 1] == v && !merged[t - 1] {
                out[t - 1] = v * 2;
                merged[t - 1] = true;
                let (tr, tc) = pos(t - 1);
                slides.push(Slide {
                    from: (r, c),
                    to: (tr, tc),
                    value: v,
                });
                merges.push((tr, tc, v * 2));
                gained += v * 2;
            } else {
                out[t] = v;
                slides.push(Slide {
                    from: (r, c),
                    to: pos(t),
                    value: v,
                });
                t += 1;
            }
        }
        for (k, &v) in out.iter().enumerate() {
            let (r, c) = pos(k);
            new_grid[r][c] = v;
        }
    }
    if new_grid == *grid {
        slides.clear();
        merges.clear();
    }
    (new_grid, gained)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Direction::*;

    fn mv(grid: Grid, d: Direction) -> (Grid, u32, Vec<(usize, usize, u32)>) {
        let (mut s, mut m) = (Vec::new(), Vec::new());
        let (g, gained) = compute_move(&grid, d, &mut s, &mut m);
        (g, gained, m)
    }

    #[test]
    fn row_merges_once_per_tile() {
        let g = [[2, 2, 2, 2], [0; 4], [0; 4], [0; 4]];
        let (n, gained, merges) = mv(g, Left);
        assert_eq!(n[0], [4, 4, 0, 0]);
        assert_eq!(gained, 8);
        assert_eq!(merges, vec![(0, 0, 4), (0, 1, 4)]);
        assert_eq!(mv(g, Right).0[0], [0, 0, 4, 4]);
    }

    #[test]
    fn no_chain_merge() {
        // [4,2,2,0] -> [4,4,0,0] et non [8,0,0,0]
        let g = [[4, 2, 2, 0], [0; 4], [0; 4], [0; 4]];
        assert_eq!(mv(g, Left).0[0], [4, 4, 0, 0]);
        // le plus proche du bord fusionne d'abord: [2,2,2,0] -> [4,2,0,0]
        let g = [[2, 2, 2, 0], [0; 4], [0; 4], [0; 4]];
        assert_eq!(mv(g, Left).0[0], [4, 2, 0, 0]);
        assert_eq!(mv(g, Right).0[0], [0, 0, 2, 4]);
    }

    #[test]
    fn slides_gaps_in_all_directions() {
        let g = [[0, 0, 0, 2], [0, 0, 0, 0], [0, 0, 0, 0], [4, 0, 0, 0]];
        assert_eq!(mv(g, Left).0, [[2, 0, 0, 0], [0; 4], [0; 4], [4, 0, 0, 0]]);
        assert_eq!(mv(g, Right).0, [[0, 0, 0, 2], [0; 4], [0; 4], [0, 0, 0, 4]]);
        assert_eq!(mv(g, Up).0, [[4, 0, 0, 2], [0; 4], [0; 4], [0; 4]]);
        assert_eq!(mv(g, Down).0, [[0; 4], [0; 4], [0; 4], [4, 0, 0, 2]]);
    }

    #[test]
    fn vertical_merges() {
        let g = [[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]];
        let (n, gained, _) = mv(g, Up);
        assert_eq!([n[0][0], n[1][0], n[2][0], n[3][0]], [4, 8, 0, 0]);
        assert_eq!(gained, 12);
        let (n, _, _) = mv(g, Down);
        assert_eq!([n[0][0], n[1][0], n[2][0], n[3][0]], [0, 0, 4, 8]);
    }

    #[test]
    fn blocked_move_changes_nothing() {
        // Aucune case libre ni fusion possible dans ce sens
        let g = [[2, 4, 8, 16], [0; 4], [0; 4], [0; 4]];
        let (mut s, mut m) = (Vec::new(), Vec::new());
        let (n, gained) = compute_move(&g, Left, &mut s, &mut m);
        assert_eq!((n, gained), (g, 0));
        assert!(s.is_empty() && m.is_empty()); // `moved` = false côté jeu
        let (n, _, _) = mv(g, Up);
        assert_eq!(n, g);
    }

    #[test]
    fn full_board_without_merge_is_stuck() {
        let g = [[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]];
        for d in [Left, Right, Up, Down] {
            assert_eq!(mv(g, d).0, g);
        }
    }

    #[test]
    fn reaching_2048_is_reported() {
        let g = [[1024, 1024, 0, 0], [0; 4], [0; 4], [0; 4]];
        assert_eq!(mv(g, Left).2, vec![(0, 0, 2048)]);
    }
}
