use crate::core::{Game, GameAction};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph},
};
use std::time::Duration;

// Tailles de grille prédéfinies
const SMALL_WIDTH: usize = 40;
const SMALL_HEIGHT: usize = 20;
const MEDIUM_WIDTH: usize = 60;
const MEDIUM_HEIGHT: usize = 30;
const LARGE_WIDTH: usize = 80;
const LARGE_HEIGHT: usize = 40;
const HUGE_WIDTH: usize = 120;
const HUGE_HEIGHT: usize = 60;

const MAX_GRID_WIDTH: usize = HUGE_WIDTH;
const MAX_GRID_HEIGHT: usize = HUGE_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Dead,
    Alive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Paused,
    Running,
    Editing,
}

#[derive(Debug, Clone, Copy)]
pub enum Pattern {
    Glider,
    Blinker,
    Block,
    Toad,
    Beacon,
    Pulsar,
}

pub struct GameOfLife {
    grid: [[CellState; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT],
    next_grid: [[CellState; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT],
    state: GameState,
    generation: u32,
    cursor_x: usize,
    cursor_y: usize,
    camera_x: usize, // Position de la caméra pour la vue
    camera_y: usize,
    speed: u8, // 1-5, plus élevé = plus rapide
    grid_width: usize,
    grid_height: usize,
}

impl GameOfLife {
    pub fn new() -> Self {
        let mut game = Self {
            grid: [[CellState::Dead; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT],
            next_grid: [[CellState::Dead; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT],
            state: GameState::Editing,
            generation: 0,
            cursor_x: MEDIUM_WIDTH / 2,
            cursor_y: MEDIUM_HEIGHT / 2,
            camera_x: MEDIUM_WIDTH / 2,
            camera_y: MEDIUM_HEIGHT / 2,
            speed: 3,
            grid_width: MEDIUM_WIDTH,
            grid_height: MEDIUM_HEIGHT,
        };
        
        // Commencer avec un pattern initial
        game.place_pattern(Pattern::Glider, game.grid_width / 2, game.grid_height / 2);
        game.place_pattern(Pattern::Blinker, game.grid_width / 2 - 10, game.grid_height / 2 - 5);
        game.place_pattern(Pattern::Block, game.grid_width / 2 + 10, game.grid_height / 2 + 5);
        
        game
    }
    
    fn resize_grid(&mut self, width: usize, height: usize) {
        // Limiter aux dimensions maximales
        let new_width = width.min(MAX_GRID_WIDTH);
        let new_height = height.min(MAX_GRID_HEIGHT);
        
        // Créer une nouvelle grille vide
        let mut new_grid = [[CellState::Dead; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT];
        
        // Copier les cellules existantes si elles rentrent dans la nouvelle taille
        for y in 0..new_height.min(self.grid_height) {
            for x in 0..new_width.min(self.grid_width) {
                new_grid[y][x] = self.grid[y][x];
            }
        }
        
        self.grid = new_grid;
        self.next_grid = [[CellState::Dead; MAX_GRID_WIDTH]; MAX_GRID_HEIGHT];
        self.grid_width = new_width;
        self.grid_height = new_height;
        
        // Ajuster la position du curseur et de la caméra
        self.cursor_x = self.cursor_x.min(new_width.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(new_height.saturating_sub(1));
        self.camera_x = self.camera_x.min(new_width.saturating_sub(1));
        self.camera_y = self.camera_y.min(new_height.saturating_sub(1));
        
        self.generation = 0;
    }
    
    fn clear_grid(&mut self) {
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                self.grid[y][x] = CellState::Dead;
            }
        }
        self.generation = 0;
    }
    
    fn randomize_grid(&mut self) {
        let mut rng = rand::rng();
        for row in 0..self.grid_height {
            for col in 0..self.grid_width {
                self.grid[row][col] = if rng.random_bool(0.3) {
                    CellState::Alive
                } else {
                    CellState::Dead
                };
            }
        }
        self.generation = 0;
    }
    
    fn place_pattern(&mut self, pattern: Pattern, start_x: usize, start_y: usize) {
        let pattern_cells = match pattern {
            Pattern::Glider => vec![
                (0, 1), (1, 2), (2, 0), (2, 1), (2, 2)
            ],
            Pattern::Blinker => vec![
                (1, 0), (1, 1), (1, 2)
            ],
            Pattern::Block => vec![
                (0, 0), (0, 1), (1, 0), (1, 1)
            ],
            Pattern::Toad => vec![
                (1, 1), (1, 2), (1, 3), (2, 0), (2, 1), (2, 2)
            ],
            Pattern::Beacon => vec![
                (0, 0), (0, 1), (1, 0), (1, 1),
                (2, 2), (2, 3), (3, 2), (3, 3)
            ],
            Pattern::Pulsar => vec![
                // Pattern complexe du pulsar (13x13)
                (2, 4), (2, 5), (2, 6), (2, 10), (2, 11), (2, 12),
                (4, 2), (4, 7), (4, 9), (4, 14),
                (5, 2), (5, 7), (5, 9), (5, 14),
                (6, 2), (6, 7), (6, 9), (6, 14),
                (7, 4), (7, 5), (7, 6), (7, 10), (7, 11), (7, 12),
                (9, 4), (9, 5), (9, 6), (9, 10), (9, 11), (9, 12),
                (10, 2), (10, 7), (10, 9), (10, 14),
                (11, 2), (11, 7), (11, 9), (11, 14),
                (12, 2), (12, 7), (12, 9), (12, 14),
                (14, 4), (14, 5), (14, 6), (14, 10), (14, 11), (14, 12),
            ],
        };
        
        for (dx, dy) in pattern_cells {
            let x = start_x + dx;
            let y = start_y + dy;
            if x < self.grid_width && y < self.grid_height {
                self.grid[y][x] = CellState::Alive;
            }
        }
    }
    
    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        
        for dy in -1..=1i32 {
            for dx in -1..=1i32 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                
                if nx >= 0 && nx < self.grid_width as i32 && ny >= 0 && ny < self.grid_height as i32 {
                    let nx = nx as usize;
                    let ny = ny as usize;
                    if self.grid[ny][nx] == CellState::Alive {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    fn update_generation(&mut self) {
        // Calculer la prochaine génération
        for y in 0..self.grid_height {
            for x in 0..self.grid_width {
                let neighbors = self.count_neighbors(x, y);
                let current_cell = self.grid[y][x];
                
                self.next_grid[y][x] = match (current_cell, neighbors) {
                    // Règle 1: Une cellule vivante avec moins de 2 voisins meurt (sous-population)
                    (CellState::Alive, n) if n < 2 => CellState::Dead,
                    // Règle 2: Une cellule vivante avec 2 ou 3 voisins survit
                    (CellState::Alive, 2..=3) => CellState::Alive,
                    // Règle 3: Une cellule vivante avec plus de 3 voisins meurt (surpopulation)
                    (CellState::Alive, n) if n > 3 => CellState::Dead,
                    // Règle 4: Une cellule morte avec exactement 3 voisins devient vivante (reproduction)
                    (CellState::Dead, 3) => CellState::Alive,
                    // Toutes les autres cellules restent dans leur état
                    (state, _) => state,
                };
            }
        }
        
        // Copier la nouvelle génération
        self.grid = self.next_grid;
        self.generation += 1;
    }
    
    fn toggle_cell(&mut self, x: usize, y: usize) {
        if x < self.grid_width && y < self.grid_height {
            self.grid[y][x] = match self.grid[y][x] {
                CellState::Alive => CellState::Dead,
                CellState::Dead => CellState::Alive,
            };
        }
    }
    
    fn step_forward(&mut self) {
        self.update_generation();
    }
    
    fn change_speed(&mut self, delta: i8) {
        self.speed = ((self.speed as i8 + delta).max(1).min(5)) as u8;
    }
    
    fn get_tick_rate(&self) -> Duration {
        match self.speed {
            1 => Duration::from_millis(1000),
            2 => Duration::from_millis(500),
            3 => Duration::from_millis(250),
            4 => Duration::from_millis(125),
            5 => Duration::from_millis(60),
            _ => Duration::from_millis(250),
        }
    }
}

impl Game for GameOfLife {
    fn name(&self) -> &'static str {
        "Game of Life"
    }
    
    fn description(&self) -> &'static str {
        "Conway's Game of Life - Cellular automaton visualization"
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match key.code {
            // Contrôles de mouvement
            KeyCode::Up | KeyCode::Char('w') => {
                match self.state {
                    GameState::Editing => {
                        if self.cursor_y > 0 {
                            self.cursor_y -= 1;
                            self.camera_y = self.cursor_y; // La caméra suit le curseur
                        }
                    }
                    _ => {
                        // En mode observation, déplacer la caméra
                        if self.camera_y > 0 {
                            self.camera_y -= 1;
                        }
                    }
                }
                GameAction::Continue
            }
            KeyCode::Down | KeyCode::Char('s') => {
                match self.state {
                    GameState::Editing => {
                        if self.cursor_y < self.grid_height - 1 {
                            self.cursor_y += 1;
                            self.camera_y = self.cursor_y;
                        }
                    }
                    _ => {
                        if self.camera_y < self.grid_height - 1 {
                            self.camera_y += 1;
                        }
                    }
                }
                GameAction::Continue
            }
            KeyCode::Left | KeyCode::Char('a') => {
                match self.state {
                    GameState::Editing => {
                        if self.cursor_x > 0 {
                            self.cursor_x -= 1;
                            self.camera_x = self.cursor_x;
                        }
                    }
                    _ => {
                        if self.camera_x > 0 {
                            self.camera_x -= 1;
                        }
                    }
                }
                GameAction::Continue
            }
            KeyCode::Right | KeyCode::Char('d') => {
                match self.state {
                    GameState::Editing => {
                        if self.cursor_x < self.grid_width - 1 {
                            self.cursor_x += 1;
                            self.camera_x = self.cursor_x;
                        }
                    }
                    _ => {
                        if self.camera_x < self.grid_width - 1 {
                            self.camera_x += 1;
                        }
                    }
                }
                GameAction::Continue
            }
            
            // Toggle cellule en mode édition
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.state == GameState::Editing {
                    self.toggle_cell(self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            
            // Contrôles de simulation
            KeyCode::Char('p') => {
                self.state = match self.state {
                    GameState::Running => GameState::Paused,
                    GameState::Paused => GameState::Running,
                    GameState::Editing => GameState::Running,
                };
                GameAction::Continue
            }
            KeyCode::Char('e') => {
                self.state = GameState::Editing;
                GameAction::Continue
            }
            KeyCode::Char('n') => {
                if self.state != GameState::Running {
                    self.step_forward();
                }
                GameAction::Continue
            }
            
            // Contrôles de vitesse
            KeyCode::Char('+') | KeyCode::Char('=') => {
                self.change_speed(1);
                GameAction::Continue
            }
            KeyCode::Char('-') => {
                self.change_speed(-1);
                GameAction::Continue
            }
            
            // Patterns prédéfinis
            KeyCode::Char('1') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Glider, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            KeyCode::Char('2') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Blinker, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            KeyCode::Char('3') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Block, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            KeyCode::Char('4') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Toad, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            KeyCode::Char('5') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Beacon, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            KeyCode::Char('6') => {
                if self.state == GameState::Editing {
                    self.place_pattern(Pattern::Pulsar, self.cursor_x, self.cursor_y);
                }
                GameAction::Continue
            }
            
            // Tailles de grille
            KeyCode::F(1) => {
                self.resize_grid(SMALL_WIDTH, SMALL_HEIGHT);
                GameAction::Continue
            }
            KeyCode::F(2) => {
                self.resize_grid(MEDIUM_WIDTH, MEDIUM_HEIGHT);
                GameAction::Continue
            }
            KeyCode::F(3) => {
                self.resize_grid(LARGE_WIDTH, LARGE_HEIGHT);
                GameAction::Continue
            }
            KeyCode::F(4) => {
                self.resize_grid(HUGE_WIDTH, HUGE_HEIGHT);
                GameAction::Continue
            }
            
            // Utilitaires
            KeyCode::Char('c') => {
                self.clear_grid();
                GameAction::Continue
            }
            KeyCode::Char('r') => {
                self.randomize_grid();
                GameAction::Continue
            }
            
            KeyCode::Char('q') => GameAction::Quit,
            _ => GameAction::Continue,
        }
    }
    
    fn update(&mut self) -> GameAction {
        if self.state == GameState::Running {
            self.update_generation();
        }
        GameAction::Continue
    }
    
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_game_of_life(frame, self);
    }
    
    fn tick_rate(&self) -> Duration {
        if self.state == GameState::Running {
            self.get_tick_rate()
        } else {
            Duration::from_millis(100)
        }
    }
}

fn draw_game_of_life(frame: &mut ratatui::Frame, game: &GameOfLife) {
    let area = frame.area();
    
    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec infos
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(4), // Footer avec instructions
    ])
    .split(area);
    
    // Fond sombre
    let background = Block::new().style(Style::default().bg(Color::Black));
    frame.render_widget(background, area);
    
    // === HEADER ===
    let state_text = match game.state {
        GameState::Running => "RUNNING".green().bold(),
        GameState::Paused => "PAUSED".yellow().bold(),
        GameState::Editing => "EDITING".cyan().bold(),
    };
    
    let header_text = vec![
        Line::from(vec![
            "🧬 ".green().bold(),
            "GAME OF LIFE".cyan().bold(),
            " 🧬".green().bold(),
        ]),
        Line::from(vec![
            "Gen: ".white(),
            format!("{}", game.generation).yellow().bold(),
            "  State: ".white(),
            state_text,
            "  Speed: ".white(),
            format!("{}/5", game.speed).green().bold(),
            "  Size: ".white(),
            format!("{}x{}", game.grid_width, game.grid_height).cyan().bold(),
        ]),
    ];
    
    let header = Paragraph::new(header_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Conway's Game of Life ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(25, 35, 45))),
        );
    frame.render_widget(header, chunks[0]);
    
    // === ZONE DE JEU ===
    let game_area = chunks[1];
    let game_block = Block::bordered()
        .title(" Cellular Automaton ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(5, 10, 15)));
    frame.render_widget(game_block, game_area);
    
    let inner_area = game_area.inner(ratatui::layout::Margin {
        vertical: 1,
        horizontal: 2,
    });
    
    // Calculer les dimensions des cellules (comme Snake)
    let cell_width = 2;  // Largeur de chaque cellule (2 caractères pour un aspect carré)
    let cell_height = 1; // Hauteur de chaque cellule
    
    // Calculer combien de cellules on peut afficher
    let cells_per_row = (inner_area.width as usize / cell_width).min(game.grid_width);
    let cells_per_col = (inner_area.height as usize / cell_height).min(game.grid_height);
    
    // Calculer l'offset pour centrer la vue sur la caméra
    let start_x = if game.grid_width > cells_per_row {
        game.camera_x.saturating_sub(cells_per_row / 2)
            .min(game.grid_width - cells_per_row)
    } else {
        0
    };
    
    let start_y = if game.grid_height > cells_per_col {
        game.camera_y.saturating_sub(cells_per_col / 2)
            .min(game.grid_height - cells_per_col)
    } else {
        0
    };
    
    // Calculer le centrage de la grille dans la zone disponible
    let total_grid_width = cells_per_row * cell_width;
    let total_grid_height = cells_per_col * cell_height;
    let grid_start_x = inner_area.x + (inner_area.width as usize).saturating_sub(total_grid_width) as u16 / 2;
    let grid_start_y = inner_area.y + (inner_area.height as usize).saturating_sub(total_grid_height) as u16 / 2;
    
    // Dessiner la grille cellule par cellule
    for display_y in 0..cells_per_col {
        for display_x in 0..cells_per_row {
            let grid_x = start_x + display_x;
            let grid_y = start_y + display_y;
            
            if grid_x >= game.grid_width || grid_y >= game.grid_height {
                continue;
            }
            
            let cell_x = grid_start_x + (display_x * cell_width) as u16;
            let cell_y = grid_start_y + display_y as u16;
            
            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: cell_width as u16,
                height: cell_height as u16,
            };
            
            // Déterminer le contenu et le style de la cellule
            let (cell_content, cell_style) = if game.state == GameState::Editing 
                && grid_x == game.cursor_x 
                && grid_y == game.cursor_y {
                // Curseur en mode édition
                match game.grid[grid_y][grid_x] {
                    CellState::Alive => ("██", Style::default().bg(Color::Yellow).fg(Color::Green).bold()),
                    CellState::Dead => ("  ", Style::default().bg(Color::Yellow)),
                }
            } else {
                // Cellule normale
                match game.grid[grid_y][grid_x] {
                    CellState::Alive => ("██", Style::default().fg(Color::Green).bold()),
                    CellState::Dead => ("  ", Style::default().bg(Color::Rgb(20, 25, 30))),
                }
            };
            
            let cell_widget = Paragraph::new(cell_content)
                .style(cell_style);
            
            frame.render_widget(cell_widget, cell_area);
        }
    }
    
    // === FOOTER ===
    let instructions = match game.state {
        GameState::Editing => vec![
            Line::from(vec![
                "↑↓←→".cyan().bold(),
                " Move  ".white(),
                "SPACE".green().bold(),
                " Toggle  ".white(),
                "P".yellow().bold(),
                " Play  ".white(),
                "N".blue().bold(),
                " Step  ".white(),
                "1-6".magenta().bold(),
                " Patterns".white(),
            ]),
            Line::from(vec![
                "F1-F4".cyan().bold(),
                " Size  ".white(),
                "C".red().bold(),
                " Clear  ".white(),
                "R".green().bold(),
                " Random  ".white(),
                "±".cyan().bold(),
                " Speed  ".white(),
                "Q".red().bold(),
                " Quit".white(),
            ]),
        ],
        GameState::Running => vec![
            Line::from(vec![
                "RUNNING".green().bold(),
                "  ".white(),
                "↑↓←→".cyan().bold(),
                " Pan  ".white(),
                "P".yellow().bold(),
                " Pause  ".white(),
                "E".cyan().bold(),
                " Edit  ".white(),
                "±".cyan().bold(),
                " Speed".white(),
            ]),
            Line::from(vec![
                "F1-F4".cyan().bold(),
                " Size  ".white(),
                "Gen: ".gray(),
                format!("{}", game.generation).yellow().bold(),
                "  Speed: ".gray(),
                format!("{}/5", game.speed).green().bold(),
                "  ".white(),
                "Q".red().bold(),
                " Quit".white(),
            ]),
        ],
        GameState::Paused => vec![
            Line::from(vec![
                "PAUSED".yellow().bold(),
                "  ".white(),
                "↑↓←→".cyan().bold(),
                " Pan  ".white(),
                "P".green().bold(),
                " Resume  ".white(),
                "E".cyan().bold(),
                " Edit  ".white(),
                "N".blue().bold(),
                " Step".white(),
            ]),
            Line::from(vec![
                "F1-F4".cyan().bold(),
                " Size  ".white(),
                "Gen: ".gray(),
                format!("{}", game.generation).yellow().bold(),
                "  Speed: ".gray(),
                format!("{}/5", game.speed).green().bold(),
                "  ".white(),
                "Q".red().bold(),
                " Quit".white(),
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
    
    // === POPUP D'AIDE PATTERNS ===
    if game.state == GameState::Editing {
        // Afficher l'aide des patterns dans un coin
        let help_width = 32;
        let help_height = 14;
        let help_area = Rect {
            x: area.width.saturating_sub(help_width),
            y: chunks[0].height,
            width: help_width,
            height: help_height,
        };
        
        let help_text = vec![
            Line::from(" Patterns:".yellow().bold()),
            Line::from(" 1 - Glider".white()),
            Line::from(" 2 - Blinker".white()),
            Line::from(" 3 - Block".white()),
            Line::from(" 4 - Toad".white()),
            Line::from(" 5 - Beacon".white()),
            Line::from(" 6 - Pulsar".white()),
            Line::from(""),
            Line::from(" Grid Sizes:".cyan().bold()),
            Line::from(" F1 - Small (40x20)".white()),
            Line::from(" F2 - Medium (60x30)".white()),
            Line::from(" F3 - Large (80x40)".white()),
            Line::from(" F4 - Huge (120x60)".white()),
        ];
        
        let help_popup = Paragraph::new(help_text)
            .block(
                Block::bordered()
                    .title(" Help ".cyan().bold())
                    .border_style(Style::new().cyan())
                    .style(Style::default().bg(Color::Rgb(20, 20, 30)))
            );
        
        frame.render_widget(help_popup, help_area);
    }
}