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

const GRID_WIDTH: usize = 16;
const GRID_HEIGHT: usize = 16;
const MINE_COUNT: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    is_mine: bool,
    adjacent_mines: u8,
    state: CellState,
}

impl Cell {
    fn new() -> Self {
        Self {
            is_mine: false,
            adjacent_mines: 0,
            state: CellState::Hidden,
        }
    }
}

pub struct MinesweeperGame {
    grid: [[Cell; GRID_WIDTH]; GRID_HEIGHT],
    cursor_x: usize,
    cursor_y: usize,
    game_over: bool,
    won: bool,
    mines_generated: bool,
    flags_used: usize,
    cells_revealed: usize,

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
    time: f32,
    show_at: [[f32; GRID_WIDTH]; GRID_HEIGHT], // instant d'apparition (onde de révélation)
    reveal_origin: (usize, usize),
    confetti: f32,      // durée restante de la pluie de confettis
    origin: (u16, u16), // coin de la grille, mis à jour au draw
}

const CELL_W: u16 = 3;
const WAVE_DELAY: f32 = 0.035; // secondes par cellule de distance

impl MinesweeperGame {
    pub fn new() -> Self {
        Self {
            grid: [[Cell::new(); GRID_WIDTH]; GRID_HEIGHT],
            cursor_x: GRID_WIDTH / 2,
            cursor_y: GRID_HEIGHT / 2,
            game_over: false,
            won: false,
            mines_generated: false,
            flags_used: 0,
            cells_revealed: 0,

            audio: AudioManager::default(),
            music_started: false,

            highscore_manager: HighScoreManager::default(),
            start_time: std::time::Instant::now(),
            score_saved: false,

            particles: Particles::new(256, 6.0),
            shake: Shake::default(),
            time: 0.0,
            show_at: [[0.0; GRID_WIDTH]; GRID_HEIGHT],
            reveal_origin: (0, 0),
            confetti: 0.0,
            origin: (0, 0),
        }
    }

    /// Centre terminal d'une cellule.
    fn cell_center(&self, x: usize, y: usize) -> (f32, f32) {
        (
            (self.origin.0 + x as u16 * CELL_W) as f32 + 1.5,
            (self.origin.1 + y as u16) as f32 + 0.5,
        )
    }

    /// Instant d'apparition d'une cellule selon sa distance au point de départ.
    fn wave_time(&self, x: usize, y: usize) -> f32 {
        let (ox, oy) = self.reveal_origin;
        self.time + (x.abs_diff(ox) as f32).hypot(y.abs_diff(oy) as f32) * WAVE_DELAY
    }

    fn generate_mines(&mut self, first_click_x: usize, first_click_y: usize) {
        if self.mines_generated {
            return;
        }

        let mut rng = rand::rng();
        let mut mines_placed = 0;

        while mines_placed < MINE_COUNT {
            let x = rng.random_range(0..GRID_WIDTH);
            let y = rng.random_range(0..GRID_HEIGHT);

            // Ne pas placer de mine sur le premier clic ou autour
            if (x.abs_diff(first_click_x) <= 1 && y.abs_diff(first_click_y) <= 1)
                || self.grid[y][x].is_mine
            {
                continue;
            }

            self.grid[y][x].is_mine = true;
            mines_placed += 1;
        }

        // Calculer les nombres adjacents
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                if !self.grid[y][x].is_mine {
                    self.grid[y][x].adjacent_mines = self.count_adjacent_mines(x, y);
                }
            }
        }

        self.mines_generated = true;
    }

    fn start_music_if_needed(&mut self) {
        if !self.music_started && self.audio.is_music_enabled() && !self.game_over && !self.won {
            // Choisir la version selon le nombre de drapeaux utilisés (indicateur de progression)
            let flag_ratio = self.flags_used as f32 / MINE_COUNT as f32;
            if flag_ratio > 0.7 {
                self.audio.play_minesweeper_music_fast(); // Version tendue pour fin de partie
            } else {
                self.audio.play_minesweeper_music(); // Version contemplative normale
            }
            self.music_started = true;
        }

        // Relancer la musique si elle est finie
        if self.music_started
            && self.audio.is_music_enabled()
            && !self.game_over
            && !self.won
            && self.audio.is_music_empty()
        {
            let flag_ratio = self.flags_used as f32 / MINE_COUNT as f32;
            if flag_ratio > 0.7 {
                self.audio.play_minesweeper_music_fast();
            } else {
                self.audio.play_minesweeper_music();
            }
        }
    }

    fn count_adjacent_mines(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;

        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.grid[ny][nx].is_mine {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    fn reveal_cell(&mut self, x: usize, y: usize) {
        self.reveal_origin = (x, y);
        self.reveal_cell_internal(x, y, true);
    }

    fn reveal_cell_internal(&mut self, x: usize, y: usize, play_sound: bool) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return;
        }

        if self.grid[y][x].state != CellState::Hidden {
            return;
        }

        if !self.mines_generated {
            self.generate_mines(x, y);
        }

        self.grid[y][x].state = CellState::Revealed;
        self.cells_revealed += 1;
        self.show_at[y][x] = self.wave_time(x, y);

        let cell = &self.grid[y][x];

        if cell.is_mine {
            self.game_over = true;
            // Son d'explosion
            self.audio.play_sound(SoundEffect::MinesweeperMineHit);
            // Révéler toutes les mines
            self.shake.kick(2.5);
            let (cx, cy) = self.cell_center(x, y);
            self.particles
                .burst(cx, cy, 50, 14.0, (255, 200, 60), (160, 20, 0));
            self.particles
                .burst(cx, cy, 30, 9.0, (150, 150, 150), (30, 30, 30));
            self.particles
                .spray(cx, cy, 20, 6.0, (255, 90, 30), (60, 60, 60), 6.0);
            for my in 0..GRID_HEIGHT {
                for mx in 0..GRID_WIDTH {
                    if self.grid[my][mx].is_mine {
                        if self.grid[my][mx].state != CellState::Revealed {
                            self.show_at[my][mx] = self.wave_time(mx, my) + 0.15;
                        }
                        self.grid[my][mx].state = CellState::Revealed;
                    }
                }
            }

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
            return;
        }

        // Son de révélation normale - seulement pour le clic initial
        if play_sound {
            self.audio.play_sound(SoundEffect::MinesweeperReveal);
        }

        // Si la case n'a pas de mines adjacentes, révéler les cases voisines
        if cell.adjacent_mines == 0 {
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                        self.reveal_cell_internal(nx as usize, ny as usize, false);
                    }
                }
            }
        }

        // Vérifier la victoire
        if self.cells_revealed == (GRID_WIDTH * GRID_HEIGHT - MINE_COUNT) {
            self.won = true;
            self.confetti = 2.5;
            // Son de victoire
            self.audio.play_sound(SoundEffect::MinesweeperVictory);
            self.audio.stop_music();
            self.audio.play_minesweeper_music_celebration();
            self.music_started = false;

            // Sauvegarder le score si c'est un high score et pas encore sauvé
            self.save_high_score_if_needed();
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if x >= GRID_WIDTH || y >= GRID_HEIGHT {
            return;
        }

        let cell = &mut self.grid[y][x];
        match cell.state {
            CellState::Hidden => {
                if self.flags_used < MINE_COUNT {
                    cell.state = CellState::Flagged;
                    self.flags_used += 1;
                    let (cx, cy) = self.cell_center(x, y);
                    self.particles
                        .burst(cx, cy, 8, 6.0, (255, 220, 60), (255, 90, 20));
                    // Son de placement de drapeau
                    self.audio.play_sound(SoundEffect::MinesweeperFlag);
                }
            }
            CellState::Flagged => {
                cell.state = CellState::Hidden;
                self.flags_used -= 1;
                // Son de retrait de drapeau
                self.audio.play_sound(SoundEffect::MinesweeperUnflag);
            }
            CellState::Revealed => {}
        }
    }

    fn restart(&mut self) {
        self.grid = [[Cell::new(); GRID_WIDTH]; GRID_HEIGHT];
        self.cursor_x = GRID_WIDTH / 2;
        self.cursor_y = GRID_HEIGHT / 2;
        self.game_over = false;
        self.won = false;
        self.mines_generated = false;
        self.flags_used = 0;
        self.cells_revealed = 0;
        self.score_saved = false;
        self.show_at = [[0.0; GRID_WIDTH]; GRID_HEIGHT];
        self.confetti = 0.0;
        self.start_time = std::time::Instant::now();

        self.audio.stop_music();
        self.music_started = false;
    }

    fn save_high_score_if_needed(&mut self) {
        // Ne sauvegarder qu'une seule fois
        if self.score_saved {
            return;
        }

        // Calculer un score basé sur le temps et les performances
        let duration = self.start_time.elapsed().as_secs();
        let base_score = if self.won {
            // Score de base élevé pour une victoire
            10000u32
        } else {
            // Score basé sur les cellules révélées pour un échec
            (self.cells_revealed as u32) * 10
        };

        // Bonus de temps (moins de temps = meilleur score)
        let time_bonus = if duration > 0 {
            (3600 / duration.max(1)) as u32 // Bonus inversement proportionnel au temps
        } else {
            3600
        };

        let final_score = base_score + time_bonus;

        // Vérifier si c'est un high score
        if self
            .highscore_manager
            .is_high_score("minesweeper", final_score)
        {
            let game_data = GameData::Minesweeper {
                grid_size: (GRID_WIDTH as u32, GRID_HEIGHT as u32),
                mines_count: MINE_COUNT as u32,
                duration_seconds: duration,
            };

            let score = Score::new("Anonymous".to_string(), final_score, game_data);

            // Sauvegarder le score
            if let Ok(_is_top_10) = self.highscore_manager.add_score("minesweeper", score) {
                self.score_saved = true;
            }
        }
    }

    fn get_cell_color(cell: &Cell) -> Color {
        match cell.state {
            CellState::Hidden => Color::Rgb(160, 160, 160),
            CellState::Flagged => Color::Rgb(255, 100, 100),
            CellState::Revealed => {
                if cell.is_mine {
                    Color::Rgb(255, 50, 50)
                } else {
                    Color::Rgb(220, 220, 220)
                }
            }
        }
    }

    fn get_cell_text_color(cell: &Cell) -> Color {
        if cell.state == CellState::Revealed && !cell.is_mine {
            match cell.adjacent_mines {
                1 => Color::Blue,
                2 => Color::Green,
                3 => Color::Red,
                4 => Color::Rgb(128, 0, 128), // Purple
                5 => Color::Rgb(128, 0, 0),   // Maroon
                6 => Color::Cyan,
                7 => Color::Black,
                8 => Color::Rgb(128, 128, 128), // Gray
                _ => Color::Black,
            }
        } else {
            Color::Black
        }
    }

    fn get_cell_text(cell: &Cell) -> String {
        match cell.state {
            CellState::Hidden => " ".to_string(),
            CellState::Flagged => "F".to_string(),
            CellState::Revealed => {
                if cell.is_mine {
                    "*".to_string()
                } else if cell.adjacent_mines > 0 {
                    cell.adjacent_mines.to_string()
                } else {
                    " ".to_string()
                }
            }
        }
    }
}

impl Game for MinesweeperGame {
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
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                    GameAction::Continue
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    if self.cursor_y < GRID_HEIGHT - 1 {
                        self.cursor_y += 1;
                    }
                    GameAction::Continue
                }
                KeyCode::Left | KeyCode::Char('a') => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                    GameAction::Continue
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    if self.cursor_x < GRID_WIDTH - 1 {
                        self.cursor_x += 1;
                    }
                    GameAction::Continue
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.reveal_cell(self.cursor_x, self.cursor_y);
                    GameAction::Continue
                }
                KeyCode::Char('f') => {
                    self.toggle_flag(self.cursor_x, self.cursor_y);
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
        self.time += dt;
        self.particles.update(dt);
        self.shake.update(dt);
        if self.confetti > 0.0 {
            self.confetti -= dt;
            // Confettis multicolores qui jaillissent vers le haut depuis le bas de la grille
            let mut rng = rand::rng();
            let x =
                self.origin.0 as f32 + rng.random_range(0.0..(GRID_WIDTH as u16 * CELL_W) as f32);
            let y = (self.origin.1 as usize + GRID_HEIGHT) as f32;
            let c: Rgb = [
                (255, 90, 90),
                (255, 220, 70),
                (90, 230, 120),
                (90, 170, 255),
                (220, 110, 255),
            ][rng.random_range(0..5)];
            self.particles
                .spray(x, y, 3, 6.0, c, fx::lerp(c, (0, 0, 0), 0.6), 14.0);
        }
    }

    fn update(&mut self) -> GameAction {
        self.start_music_if_needed();
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_minesweeper_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(100)
    }
}

fn draw_minesweeper_game(frame: &mut ratatui::Frame, game: &mut MinesweeperGame) {
    let area = frame.area();

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec infos
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(4), // Footer avec instructions
    ])
    .split(area);

    // Fond sombre élégant
    let background = Block::new().style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let mines_left = MINE_COUNT.saturating_sub(game.flags_used);
    let header_text = vec![
        Line::from(vec![
            "💣 ".yellow().bold(),
            "MINESWEEPER".cyan().bold(),
            " 💣".yellow().bold(),
        ]),
        Line::from(vec![
            "Mines Left: ".yellow(),
            format!("{mines_left}").white().bold(),
            " | Flags Used: ".gray(),
            format!("{}", game.flags_used).red().bold(),
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
        .title(" Mine Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    frame.render_widget(game_block, game_area);

    let inner_area = game_area.inner(Margin {
        vertical: 1,
        horizontal: 2,
    });

    // Calculer les dimensions pour centrer la grille
    let cell_width = 3;
    let cell_height = 1;
    let grid_width = GRID_WIDTH as u16 * cell_width;
    let grid_height = GRID_HEIGHT as u16 * cell_height;

    let start_x = inner_area.x + (inner_area.width.saturating_sub(grid_width)) / 2;
    let start_y = inner_area.y + (inner_area.height.saturating_sub(grid_height)) / 2;
    game.origin = (start_x, start_y);
    let (sx, sy) = game.shake.offset();
    let start_x =
        (start_x as i32 + sx).clamp(0, area.width.saturating_sub(grid_width) as i32) as u16;
    let start_y =
        (start_y as i32 + sy).clamp(0, area.height.saturating_sub(grid_height) as i32) as u16;

    // Dessiner la grille
    for row in 0..GRID_HEIGHT {
        for col in 0..GRID_WIDTH {
            let mut cell = game.grid[row][col];
            if game.time < game.show_at[row][col] {
                cell.state = CellState::Hidden; // l'onde n'est pas encore arrivée
            }
            let cell = &cell;

            let cell_x = start_x + (col as u16 * cell_width);
            let cell_y = start_y + (row as u16 * cell_height);

            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: cell_width,
                height: cell_height,
            };

            let cell_text = MinesweeperGame::get_cell_text(cell);
            let cell_color = MinesweeperGame::get_cell_color(cell);
            let text_color = MinesweeperGame::get_cell_text_color(cell);

            // Mettre en surbrillance la case sous le curseur
            let mut style = Style::default().bg(cell_color);
            if col == game.cursor_x && row == game.cursor_y {
                style = style.bg(Color::Yellow);
            }

            let cell_widget = Paragraph::new(cell_text)
                .alignment(ratatui::layout::Alignment::Center)
                .style(style.fg(text_color).bold());

            frame.render_widget(cell_widget, cell_area);
        }
    }

    game.particles.draw(frame.buffer_mut(), (0, 0));

    // === FOOTER ===
    let instructions = if game.game_over || game.won {
        vec![
            Line::from(vec![
                if game.won {
                    "🎉 YOU WON! 🎉".green().bold()
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
        ]
    } else {
        vec![
            Line::from(vec![
                "↑↓←→".cyan().bold(),
                " Move  ".white(),
                "SPACE".cyan().bold(),
                " Reveal  ".white(),
                "F".yellow().bold(),
                " Flag  ".white(),
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
            Line::from("You hit a mine!".white()),
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
    // === VICTORY POPUP ===
    else if game.won {
        let popup_width = 50.min(area.width);
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
            Line::from("All mines found!".white()),
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
