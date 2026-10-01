use crate::audio::{AudioManager, SoundEffect};
use crate::core::{Game, GameAction};
use crate::engine::{
    fx::{self, Rgb, Shake},
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

// Taille de la grille standard Tetris
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PieceType {
    I, // Ligne
    O, // Carré
    T, // T
    S, // S
    Z, // Z
    J, // J
    L, // L
}

impl PieceType {
    fn get_shape(&self) -> &'static [&'static [bool]] {
        match self {
            PieceType::I => &[
                &[false, false, false, false],
                &[true, true, true, true],
                &[false, false, false, false],
                &[false, false, false, false],
            ],
            PieceType::O => &[&[true, true], &[true, true]],
            PieceType::T => &[
                &[false, true, false],
                &[true, true, true],
                &[false, false, false],
            ],
            PieceType::S => &[
                &[false, true, true],
                &[true, true, false],
                &[false, false, false],
            ],
            PieceType::Z => &[
                &[true, true, false],
                &[false, true, true],
                &[false, false, false],
            ],
            PieceType::J => &[
                &[true, false, false],
                &[true, true, true],
                &[false, false, false],
            ],
            PieceType::L => &[
                &[false, false, true],
                &[true, true, true],
                &[false, false, false],
            ],
        }
    }

    fn get_color(&self) -> Rgb {
        match self {
            PieceType::I => (60, 220, 230),
            PieceType::O => (245, 220, 70),
            PieceType::T => (190, 80, 220),
            PieceType::S => (80, 220, 100),
            PieceType::Z => (235, 70, 70),
            PieceType::J => (80, 110, 240),
            PieceType::L => (255, 165, 0), // Orange
        }
    }

    fn random() -> Self {
        let mut rng = crate::engine::rng::game_rng();
        match rng.random_range(0..7) {
            0 => PieceType::I,
            1 => PieceType::O,
            2 => PieceType::T,
            3 => PieceType::S,
            4 => PieceType::Z,
            5 => PieceType::J,
            _ => PieceType::L,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    piece_type: PieceType,
    position: Position,
    rotation: usize,
}

impl Piece {
    fn new(piece_type: PieceType) -> Self {
        Self {
            piece_type,
            position: Position { x: 4, y: 0 }, // Centre en haut
            rotation: 0,
        }
    }

    /// Les 4 blocs en coordonnées plateau. Rotation horaire appliquée aux coordonnées
    /// de la forme (pas d'allocation).
    fn get_blocks(&self) -> [Position; 4] {
        let shape = self.piece_type.get_shape();
        let (mut w, mut h) = (shape[0].len() as i32, shape.len() as i32);
        let mut cells = [(0i32, 0i32); 4];
        let mut n = 0;
        for (y, row) in shape.iter().enumerate() {
            for (x, &filled) in row.iter().enumerate() {
                if filled && n < 4 {
                    cells[n] = (x as i32, y as i32);
                    n += 1;
                }
            }
        }
        for _ in 0..self.rotation {
            for c in &mut cells {
                *c = (h - 1 - c.1, c.0);
            }
            std::mem::swap(&mut w, &mut h);
        }
        cells.map(|(x, y)| Position {
            x: self.position.x + x,
            y: self.position.y + y,
        })
    }

    fn moved(&self, dx: i32, dy: i32) -> Self {
        let mut piece = self.clone();
        piece.position.x += dx;
        piece.position.y += dy;
        piece
    }

    fn rotated(&self) -> Self {
        let mut piece = self.clone();
        piece.rotation = (piece.rotation + 1) % 4;
        piece
    }
}

pub struct TetrisGame {
    board: [[Option<PieceType>; BOARD_WIDTH]; BOARD_HEIGHT],
    current_piece: Option<Piece>,
    next_piece: PieceType,
    score: u32,
    lines_cleared: u32,
    level: u32,
    game_over: bool,
    drop_timer: u32,
    audio: AudioManager,
    music_started: bool,
    tetris_celebration: u32, // Compteur pour afficher "TETRIS!" à l'écran
    highscore_manager: HighScoreManager,
    start_time: std::time::Instant,
    score_saved: bool,

    // Effets
    particles: Particles,
    shake: Shake,
    fade: f32,
    origin: (u16, u16), // coin du plateau (cellules terminal), mis à jour au draw
    clear_rows: [usize; 4],
    clear_n: usize, // > 0 pendant l'animation de flash des lignes
    clear_t: f32,
    flash: [(i32, i32); 4], // blocs du dernier hard drop
    flash_t: f32,
}

impl TetrisGame {
    pub fn new() -> Self {
        let mut game = Self {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: None,
            next_piece: PieceType::random(),
            score: 0,
            lines_cleared: 0,
            level: 1,
            game_over: false,
            drop_timer: 0,
            audio: AudioManager::default(),
            music_started: false,
            tetris_celebration: 0,
            highscore_manager: HighScoreManager::default(),
            start_time: std::time::Instant::now(),
            score_saved: false,
            particles: Particles::new(256, 14.0),
            shake: Shake::default(),
            fade: 0.0,
            origin: (0, 0),
            clear_rows: [0; 4],
            clear_n: 0,
            clear_t: 0.0,
            flash: [(0, 0); 4],
            flash_t: 0.0,
        };
        game.spawn_piece();
        game
    }

    /// Centre terminal d'une cellule du plateau (2 caractères de large).
    fn cell_center(&self, x: usize, y: usize) -> (f32, f32) {
        (
            (self.origin.0 as usize + x * 2) as f32 + 1.0,
            (self.origin.1 as usize + y) as f32 + 0.5,
        )
    }

    fn spawn_piece(&mut self) {
        let new_piece = Piece::new(self.next_piece);
        self.next_piece = PieceType::random();

        if self.is_valid_position(&new_piece) {
            self.current_piece = Some(new_piece);
        } else {
            self.game_over = true;
            self.audio.stop_music();
            self.audio.play_sound(SoundEffect::TetrisGameOver);
            // Effondrement: chaque bloc se disperse en poussière grise
            for y in 0..BOARD_HEIGHT {
                for x in 0..BOARD_WIDTH {
                    if self.board[y][x].is_some() {
                        let (px, py) = self.cell_center(x, y);
                        self.particles
                            .burst(px, py, 1, 7.0, (170, 170, 170), (25, 25, 25));
                    }
                }
            }
            self.shake.kick(2.0);

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
        }
    }

    fn is_valid_position(&self, piece: &Piece) -> bool {
        fits(&self.board, piece)
    }

    fn place_piece(&mut self) {
        if let Some(piece) = &self.current_piece {
            for block in piece.get_blocks() {
                if block.y >= 0 {
                    self.board[block.y as usize][block.x as usize] = Some(piece.piece_type);
                }
            }
        }
        self.current_piece = None;

        // Jouer le son de pièce posée
        self.audio.play_sound(SoundEffect::TetrisPieceDrop);

        self.clear_lines();
        if self.clear_n == 0 {
            self.spawn_piece();
        }
    }

    fn clear_lines(&mut self) {
        let (rows, count) = full_rows(&self.board);
        let lines_to_clear = &rows[..count];

        // Jouer le son approprié selon le nombre de lignes
        if !lines_to_clear.is_empty() {
            match lines_to_clear.len() {
                1..=3 => self.audio.play_sound(SoundEffect::TetrisLineClear),
                4 => {
                    self.audio.play_sound(SoundEffect::TetrisTetris); // TETRIS!
                    self.tetris_celebration = 120; // Afficher "TETRIS!" pendant 120 frames
                                                   // Jouer une version spéciale de la musique pour célébrer
                    if self.audio.is_music_enabled() {
                        self.audio.stop_music();
                        self.audio.play_tetris_music_harmony();
                        self.music_started = false; // Pour que la musique normale reprenne après
                    }
                }
                _ => {}
            }
        }

        // Flash + particules; la suppression réelle a lieu dans `finish_clear`
        let n = lines_to_clear.len();
        if n > 0 {
            let (per, speed, lift, kick) = match n {
                1 | 2 => (2, 6.0, 0.0, 0.3),
                3 => (3, 8.0, 3.0, 1.0),
                _ => (4, 12.0, 10.0, 2.2),
            };
            for &y in lines_to_clear {
                for x in 0..BOARD_WIDTH {
                    if let Some(t) = self.board[y][x] {
                        let c = t.get_color();
                        let (px, py) = self.cell_center(x, y);
                        self.particles.spray(
                            px,
                            py,
                            per,
                            speed,
                            c,
                            fx::lerp(c, (0, 0, 0), 0.8),
                            lift,
                        );
                    }
                }
            }
            self.shake.kick(kick);
            self.clear_n = n.min(4);
            self.clear_rows[..self.clear_n].copy_from_slice(&lines_to_clear[..self.clear_n]);
            self.clear_t = 0.0;
        }

        // Mettre à jour le score et le niveau
        let lines_count = lines_to_clear.len() as u32;
        if lines_count > 0 {
            self.lines_cleared += lines_count;
            self.level = (self.lines_cleared / 10) + 1;

            // Système de score Tetris classique
            let line_score = match lines_count {
                1 => 40,
                2 => 100,
                3 => 300,
                4 => 1200, // Tetris!
                _ => 0,
            };
            self.score += line_score * self.level;
        }
    }

    fn finish_clear(&mut self) {
        collapse(&mut self.board, &self.clear_rows[..self.clear_n]);
        self.clear_n = 0;
        self.spawn_piece();
    }

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(piece) = &self.current_piece {
            let new_piece = piece.moved(dx, dy);
            if self.is_valid_position(&new_piece) {
                self.current_piece = Some(new_piece);

                // Son subtil pour le déplacement horizontal
                if dx != 0 {
                    self.audio.play_sound(SoundEffect::TetrisMove);
                }
                return true;
            }
        }
        false
    }

    fn rotate_piece(&mut self) -> bool {
        if let Some(piece) = &self.current_piece {
            let rotated_piece = piece.rotated();
            if self.is_valid_position(&rotated_piece) {
                self.current_piece = Some(rotated_piece);
                self.audio.play_sound(SoundEffect::TetrisRotate);
                return true;
            }
        }
        false
    }

    fn drop_piece(&mut self) {
        if !self.move_piece(0, 1) {
            self.place_piece();
        }
    }

    fn hard_drop(&mut self) {
        let mut dropped_lines = 0;
        while self.move_piece(0, 1) {
            dropped_lines += 1;
        }

        if dropped_lines > 0 {
            self.score += dropped_lines as u32 * 2; // Points bonus pour hard drop
            self.audio.play_sound(SoundEffect::TetrisHardDrop);
            self.shake.kick(0.4);
        }
        if let Some(p) = &self.current_piece {
            for (f, b) in self.flash.iter_mut().zip(p.get_blocks()) {
                *f = (b.x, b.y);
            }
            self.flash_t = 1.0;
        }

        self.place_piece();
    }

    fn get_drop_interval(&self) -> u32 {
        // Vitesse progressive basée sur le niveau
        std::cmp::max(1, 21 - self.level)
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() {
            // Choisir la version de la musique selon le niveau
            if self.level >= 7 {
                self.audio.play_tetris_music_fast(); // Version rapide pour les niveaux élevés
            } else {
                self.audio.play_tetris_music(); // Version normale
            }
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started && self.audio.is_music_enabled() && self.audio.is_music_empty() {
            // Choisir la version appropriée selon le niveau actuel
            if self.level >= 7 {
                self.audio.play_tetris_music_fast();
            } else {
                self.audio.play_tetris_music();
            }
        }
    }

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // Vérifier si c'est un high score
        if self.highscore_manager.is_high_score("tetris", self.score) {
            let duration = self.start_time.elapsed().as_secs();
            let game_data = GameData::Tetris {
                level: self.level,
                lines_cleared: self.lines_cleared,
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), self.score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("tetris", score) {
                self.score_saved = true;
            }
        }
    }
}

impl Game for TetrisGame {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        if self.clear_n > 0 && key.code != KeyCode::Char('q') {
            return GameAction::Continue; // animation de lignes en cours
        }
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
                KeyCode::Left => {
                    self.move_piece(-1, 0);
                    GameAction::Continue
                }
                KeyCode::Right => {
                    self.move_piece(1, 0);
                    GameAction::Continue
                }
                KeyCode::Down => {
                    // Soft drop : juste déplacer d'une case vers le bas
                    if self.move_piece(0, 1) {
                        self.score += 1; // Petit bonus pour soft drop
                    } else {
                        // Si on ne peut pas bouger, placer la pièce
                        self.place_piece();
                    }
                    GameAction::Continue
                }
                KeyCode::Up => {
                    self.rotate_piece();
                    GameAction::Continue
                }
                KeyCode::Char(' ') => {
                    self.hard_drop();
                    GameAction::Continue
                }
                KeyCode::Char('m') => {
                    // Toggle music
                    self.audio.toggle_music();
                    if self.audio.is_music_enabled() {
                        self.audio.play_tetris_music();
                        self.music_started = true;
                    } else {
                        self.music_started = false;
                    }
                    GameAction::Continue
                }
                KeyCode::Char('n') => {
                    // Toggle sound effects
                    self.audio.toggle_enabled();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            }
        }
    }

    fn update(&mut self) -> GameAction {
        if !self.game_over && self.clear_n == 0 {
            // Décrémenter le compteur de célébration
            if self.tetris_celebration > 0 {
                self.tetris_celebration -= 1;
            }

            // Démarrer la musique si ce n'est pas encore fait
            self.start_music_if_needed();

            self.drop_timer += 1;
            if self.drop_timer >= self.get_drop_interval() {
                self.drop_piece();
                self.drop_timer = 0;
            }
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_tetris_game(frame, self);
    }

    fn frame_time(&self) -> Option<Duration> {
        Some(Duration::from_millis(16))
    }

    fn animate(&mut self, dt: Duration) {
        let dt = dt.as_secs_f32().min(0.1);
        self.particles.update(dt);
        self.shake.update(dt);
        self.flash_t = (self.flash_t - dt / 0.15).max(0.0);
        if self.clear_n > 0 {
            self.clear_t += dt;
            if self.clear_t >= 0.08 {
                self.finish_clear();
            }
        }
        if self.game_over {
            self.fade = (self.fade + dt).min(1.0);
        }
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(50) // Plus rapide pour une meilleure réactivité
    }
}

fn draw_tetris_game(frame: &mut ratatui::Frame, game: &mut TetrisGame) {
    let area = frame.area();

    // Vérification de taille minimale pour éviter les erreurs de rendu
    if area.width < 30 || area.height < 15 {
        // Afficher un message d'erreur si l'écran est trop petit
        let error_text = vec![
            Line::from("Terminal too small!".red().bold()),
            Line::from("Minimum size: 30x15".yellow()),
            Line::from(format!("Current: {}x{}", area.width, area.height).gray()),
        ];

        let error_msg = Paragraph::new(error_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(
                Block::bordered()
                    .title("Error")
                    .border_style(Style::new().red()),
            );

        frame.render_widget(error_msg, area);
        return;
    }

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(4), // Footer
    ])
    .split(area);

    // Fond sombre
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let audio_status = if game.audio.is_enabled() {
        "🔊"
    } else {
        "🔇"
    };
    let music_status = if game.audio.is_music_enabled() {
        "🎵"
    } else {
        "🔇"
    };
    let speed_indicator = if game.level >= 7 { "⚡" } else { "🐌" };

    let header_text = if game.tetris_celebration > 0 {
        vec![
            Line::from(vec![
                "🧩 ".blue().bold(),
                "TETRIS".cyan().bold(),
                " 🧩  🎉 ".blue().bold(),
                "TETRIS!".yellow().bold(),
                " 🎉".blue().bold(),
            ]),
            Line::from(vec![
                "Score: ".yellow(),
                format!("{}", game.score).white().bold(),
                " | Lines: ".gray(),
                format!("{}", game.lines_cleared).green().bold(),
                " | Level: ".gray(),
                format!("{}", game.level).red().bold(),
                " ".white(),
                speed_indicator.white(),
                " | Audio: ".gray(),
                audio_status.white(),
                " | Music: ".gray(),
                music_status.white(),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![
                "🧩 ".blue().bold(),
                "TETRIS".cyan().bold(),
                " 🧩".blue().bold(),
            ]),
            Line::from(vec![
                "Score: ".yellow(),
                format!("{}", game.score).white().bold(),
                " | Lines: ".gray(),
                format!("{}", game.lines_cleared).green().bold(),
                " | Level: ".gray(),
                format!("{}", game.level).red().bold(),
                " ".white(),
                speed_indicator.white(),
                " | Audio: ".gray(),
                audio_status.white(),
                " | Music: ".gray(),
                music_status.white(),
            ]),
        ]
    };

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
    let inner_area = game_area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    // Calculer les dimensions pour centrer le jeu
    let board_width = BOARD_WIDTH as u16 * 2; // 2 caractères par bloc
    let board_height = BOARD_HEIGHT as u16;

    let game_rect = Rect {
        x: inner_area.x + (inner_area.width.saturating_sub(board_width + 20)) / 2,
        y: inner_area.y,
        width: board_width + 20, // +20 pour les infos à côté
        height: (board_height + 2).min(inner_area.height), // +2 pour les bordures, mais limité par l'écran
    };

    // Dessiner le cadre de jeu
    let game_block = Block::bordered()
        .title(" Playing Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    frame.render_widget(game_block, game_rect);

    let board_area = Rect {
        x: game_rect.x + 1,
        y: game_rect.y + 1,
        width: board_width,
        height: (BOARD_HEIGHT as u16).min(game_rect.height.saturating_sub(2)), // Limiter par l'espace disponible
    };

    game.origin = (board_area.x, board_area.y);
    let (sx, sy) = game.shake.offset();
    let (ox, oy) = (board_area.x as i32 + sx, board_area.y as i32 + sy);
    let buf = frame.buffer_mut();

    // Plateau (les lignes en cours de suppression flashent en blanc)
    for y in 0..BOARD_HEIGHT {
        let clearing = game.clear_rows[..game.clear_n].contains(&y);
        for x in 0..BOARD_WIDTH {
            let (ch, c) = match game.board[y][x] {
                Some(_) if clearing => ('█', (255, 255, 255)),
                Some(t) => ('█', t.get_color()),
                None => ('░', (40, 40, 50)),
            };
            for dx in 0..2 {
                fx::put(buf, ox + x as i32 * 2 + dx, oy + y as i32, ch, c);
            }
        }
    }

    // Flash du hard drop sur les blocs qui viennent d'être verrouillés
    if game.flash_t > 0.0 {
        for &(bx, by) in &game.flash {
            if (0..BOARD_WIDTH as i32).contains(&bx) && (0..BOARD_HEIGHT as i32).contains(&by) {
                if let Some(t) = game.board[by as usize][bx as usize] {
                    let c = fx::lerp(t.get_color(), (255, 255, 255), game.flash_t);
                    for dx in 0..2 {
                        fx::put(buf, ox + bx * 2 + dx, oy + by, '█', c);
                    }
                }
            }
        }
    }

    // Pièce active: ghost, lueur, puis la pièce
    if let Some(piece) = &game.current_piece {
        let rgb = piece.piece_type.get_color();
        let ghost = ghost_of(&game.board, piece);
        let show_ghost = fx::ghost_piece();
        let blocks = piece.get_blocks();
        let dim = fx::lerp(rgb, (10, 15, 20), 0.45);
        for b in ghost.get_blocks() {
            if show_ghost && b.y >= 0 && !blocks.contains(&b) {
                for dx in 0..2 {
                    fx::put(buf, ox + b.x * 2 + dx, oy + b.y, '·', dim);
                }
            }
        }
        let n = blocks.len().max(1) as i32;
        let (cx, cy) = (
            blocks.iter().map(|b| b.x).sum::<i32>() * 2 / n + 1,
            blocks.iter().map(|b| b.y).sum::<i32>() / n,
        );
        fx::glow(buf, ox + cx, oy + cy, 4, rgb, 0.3);
        for b in blocks.iter().filter(|b| b.y >= 0) {
            for dx in 0..2 {
                fx::put(buf, ox + b.x * 2 + dx, oy + b.y, '█', rgb);
            }
        }
    }

    game.particles.draw(buf, (sx, sy));

    if game.game_over {
        fx::fade_to_black(buf, board_area, game.fade * 0.85);
    }

    // Dessiner les infos à côté (prochaine pièce)
    let info_area = Rect {
        x: board_area.x + board_width + 2,
        y: board_area.y,
        width: game_rect.width.saturating_sub(board_width + 3),
        height: 8,
    };

    if info_area.width > 0 {
        let next_text = vec![Line::from("Next:".yellow().bold()), Line::from("")];

        let next_info = Paragraph::new(next_text).block(
            Block::bordered()
                .title(" Next ".yellow())
                .border_style(Style::new().yellow()),
        );
        frame.render_widget(next_info, info_area);

        // Dessiner la prochaine pièce
        let next_shape = game.next_piece.get_shape();
        for (y, row) in next_shape.iter().enumerate() {
            for (x, &filled) in row.iter().enumerate() {
                if filled {
                    let piece_x = info_area.x + 2 + (x as u16 * 2);
                    let piece_y = info_area.y + 3 + y as u16;

                    if piece_x + 1 < info_area.x + info_area.width
                        && piece_y < info_area.y + info_area.height
                    {
                        let piece_area = Rect {
                            x: piece_x,
                            y: piece_y,
                            width: 2,
                            height: 1,
                        };

                        let piece_cell = Paragraph::new("██")
                            .style(Style::default().fg(fx::color(game.next_piece.get_color())));
                        frame.render_widget(piece_cell, piece_area);
                    }
                }
            }
        }
    }

    // === FOOTER ===
    let instructions = vec![
        Line::from(vec![
            "←→".cyan().bold(),
            " Move  ".white(),
            "↓".green().bold(),
            " Soft Drop  ".white(),
            "↑".yellow().bold(),
            " Rotate  ".white(),
            "Space".magenta().bold(),
            " Hard Drop".white(),
        ]),
        Line::from(vec![
            "M".blue().bold(),
            " Music  ".white(),
            "N".blue().bold(),
            " Audio  ".white(),
            "Q".red().bold(),
            " Quit  ".white(),
            if game.game_over {
                "R".green().bold()
            } else {
                "".white()
            },
            if game.game_over { " Restart" } else { "" }.white(),
        ]),
    ];

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
                "Lines Cleared: ".white(),
                format!("{}", game.lines_cleared).green().bold(),
            ]),
            Line::from(vec![
                "Level Reached: ".white(),
                format!("{}", game.level).red().bold(),
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

type Board = [[Option<PieceType>; BOARD_WIDTH]; BOARD_HEIGHT];

/// La pièce est dans le plateau et ne chevauche aucun bloc posé (au-dessus du plateau: ok).
fn fits(board: &Board, piece: &Piece) -> bool {
    piece.get_blocks().iter().all(|b| {
        b.x >= 0
            && b.x < BOARD_WIDTH as i32
            && b.y < BOARD_HEIGHT as i32
            && (b.y < 0 || board[b.y as usize][b.x as usize].is_none())
    })
}

/// Position de la pièce après une chute complète (ghost piece). Sans allocation.
fn ghost_of(board: &Board, piece: &Piece) -> Piece {
    let mut ghost = piece.clone();
    while fits(board, &ghost.moved(0, 1)) {
        ghost = ghost.moved(0, 1);
    }
    ghost
}

/// Lignes complètes (indices croissants) et leur nombre; au plus 4 par pièce posée.
fn full_rows(board: &Board) -> ([usize; 4], usize) {
    let mut rows = [0; 4];
    let mut n = 0;
    for (y, row) in board.iter().enumerate() {
        if n < 4 && row.iter().all(|c| c.is_some()) {
            rows[n] = y;
            n += 1;
        }
    }
    (rows, n)
}

/// Supprime les lignes données (indices croissants) et fait tomber ce qui est au-dessus.
/// Dans l'ordre croissant: supprimer par le bas décalerait les lignes restantes à supprimer.
fn collapse(board: &mut Board, rows: &[usize]) {
    for &line in rows {
        for y in (1..=line).rev() {
            board[y] = board[y - 1];
        }
        board[0] = [None; BOARD_WIDTH];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // L'allocateur compteur est limité à Unix: sous Windows (CI, release) un allocateur global
    // utilisant du TLS plante avec STATUS_ACCESS_VIOLATION dans tout le binaire de test.
    #[cfg(unix)]
    use std::alloc::{GlobalAlloc, Layout, System};
    #[cfg(unix)]
    use std::cell::Cell;

    /// Compte les allocations du thread courant (tests uniquement).
    #[cfg(unix)]
    struct Counting;
    #[cfg(unix)]
    thread_local!(static ALLOCS: Cell<usize> = const { Cell::new(0) });
    #[cfg(unix)]
    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            let _ = ALLOCS.try_with(|c| c.set(c.get() + 1));
            System.alloc(l)
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            System.dealloc(p, l)
        }
    }
    #[cfg(unix)]
    #[global_allocator]
    static A: Counting = Counting;

    const ALL: [PieceType; 7] = [
        PieceType::I,
        PieceType::O,
        PieceType::T,
        PieceType::S,
        PieceType::Z,
        PieceType::J,
        PieceType::L,
    ];

    fn empty() -> Board {
        [[None; BOARD_WIDTH]; BOARD_HEIGHT]
    }

    fn fill(board: &mut Board, y: usize) {
        board[y] = [Some(PieceType::I); BOARD_WIDTH];
    }

    #[test]
    fn rotations_keep_four_distinct_blocks() {
        for t in ALL {
            let mut p = Piece::new(t);
            for _ in 0..4 {
                let b = p.get_blocks();
                for i in 0..4 {
                    for j in i + 1..4 {
                        assert_ne!((b[i].x, b[i].y), (b[j].x, b[j].y), "{t:?}");
                    }
                }
                p = p.rotated();
            }
            // 4 rotations = identité
            assert_eq!(p.rotation, 0);
        }
    }

    #[test]
    fn detects_full_rows() {
        let mut b = empty();
        fill(&mut b, 17);
        fill(&mut b, 19);
        b[18][3] = Some(PieceType::T); // ligne incomplète entre les deux
        let (rows, n) = full_rows(&b);
        assert_eq!(&rows[..n], &[17, 19]);
        assert_eq!(full_rows(&empty()).1, 0);
    }

    #[test]
    fn collapse_removes_adjacent_rows() {
        let mut b = empty();
        b[17][0] = Some(PieceType::T); // marqueur au-dessus des lignes pleines
        fill(&mut b, 18);
        fill(&mut b, 19);
        let (rows, n) = full_rows(&b);
        collapse(&mut b, &rows[..n]);
        assert_eq!(full_rows(&b).1, 0, "une ligne pleine a survécu");
        assert_eq!(b[19][0], Some(PieceType::T));
        assert!(b[18].iter().all(|c| c.is_none()));
    }

    #[test]
    fn collapse_keeps_rows_between_cleared_ones() {
        let mut b = empty();
        fill(&mut b, 16);
        b[17][5] = Some(PieceType::S);
        fill(&mut b, 18);
        b[19][1] = Some(PieceType::Z);
        let (rows, n) = full_rows(&b);
        collapse(&mut b, &rows[..n]);
        assert_eq!(b[19][1], Some(PieceType::Z));
        assert_eq!(b[18][5], Some(PieceType::S));
        assert_eq!(full_rows(&b).1, 0);
        assert_eq!(b[17].iter().filter(|c| c.is_some()).count(), 0);
    }

    #[test]
    fn fits_respects_walls_floor_and_blocks() {
        let b = empty();
        let mut p = Piece::new(PieceType::O);
        assert!(fits(&b, &p));
        p.position.x = -5;
        assert!(!fits(&b, &p));
        p.position.x = BOARD_WIDTH as i32;
        assert!(!fits(&b, &p));
        p.position = Position {
            x: 4,
            y: BOARD_HEIGHT as i32,
        };
        assert!(!fits(&b, &p));
        let mut b = empty();
        p.position = Position { x: 3, y: 4 };
        assert!(fits(&b, &p));
        let hit = p.get_blocks()[0];
        b[hit.y as usize][hit.x as usize] = Some(PieceType::I);
        assert!(!fits(&b, &p));
    }

    #[test]
    fn ghost_lands_on_floor_and_on_stack() {
        for t in ALL {
            let b = empty();
            let p = Piece::new(t);
            let g = ghost_of(&b, &p);
            assert!(fits(&b, &g));
            assert!(!fits(&b, &g.moved(0, 1)), "{t:?} ne touche pas le sol");
        }
        let mut b = empty();
        fill(&mut b, 19);
        let g = ghost_of(&b, &Piece::new(PieceType::I));
        assert!(g.get_blocks().iter().all(|k| k.y <= 18));
    }

    #[cfg(unix)]
    #[test]
    fn ghost_and_collision_do_not_allocate() {
        let mut b = empty();
        fill(&mut b, 19);
        let p = Piece::new(PieceType::T);
        let before = ALLOCS.with(|c| c.get());
        for _ in 0..100 {
            let g = ghost_of(&b, &p);
            assert!(fits(&b, &g));
            let _ = full_rows(&b);
            for k in g.get_blocks() {
                std::hint::black_box(k);
            }
        }
        assert_eq!(ALLOCS.with(|c| c.get()), before);
    }
}
