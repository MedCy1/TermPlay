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
}

impl Game2048 {
    pub fn new() -> Self {
        let mut game = Self {
            grid: [[0; GRID_SIZE]; GRID_SIZE],
            score: 0,
            best_score: 0, // TODO: charger depuis un fichier
            game_over: false,
            won: false,
            moved: false,
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
        
        let mut rng = rand::rng();
        let &(row, col) = empty_cells.choose(&mut rng).unwrap();
        
        // 90% chance pour 2, 10% chance pour 4
        let value = if rng.random_bool(0.9) { 2 } else { 4 };
        self.grid[row][col] = value;
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
    
    fn move_tiles(&mut self, direction: Direction) {
        self.moved = false;
        let mut new_grid = self.grid;
        
        match direction {
            Direction::Left => {
                for row in 0..GRID_SIZE {
                    let mut line: Vec<u32> = new_grid[row].iter().filter(|&&x| x != 0).cloned().collect();
                    
                    // Fusionner les tuiles adjacentes identiques
                    let mut merged_line = Vec::new();
                    let mut i = 0;
                    while i < line.len() {
                        if i + 1 < line.len() && line[i] == line[i + 1] {
                            let merged_value = line[i] * 2;
                            merged_line.push(merged_value);
                            self.score += merged_value;
                            if merged_value == 2048 && !self.won {
                                self.won = true;
                            }
                            i += 2; // Skip both tiles
                        } else {
                            merged_line.push(line[i]);
                            i += 1;
                        }
                    }
                    line = merged_line;
                    
                    // Remplir avec des zéros
                    line.resize(GRID_SIZE, 0);
                    
                    // Vérifier si quelque chose a changé
                    let new_row: [u32; GRID_SIZE] = line.as_slice().try_into().unwrap();
                    if new_grid[row] != new_row {
                        self.moved = true;
                    }
                    
                    new_grid[row] = new_row;
                }
            }
            Direction::Right => {
                for row in 0..GRID_SIZE {
                    let mut line: Vec<u32> = new_grid[row].iter().filter(|&&x| x != 0).cloned().collect();
                    line.reverse();
                    
                    // Fusionner les tuiles adjacentes identiques
                    let mut merged_line = Vec::new();
                    let mut i = 0;
                    while i < line.len() {
                        if i + 1 < line.len() && line[i] == line[i + 1] {
                            let merged_value = line[i] * 2;
                            merged_line.push(merged_value);
                            self.score += merged_value;
                            if merged_value == 2048 && !self.won {
                                self.won = true;
                            }
                            i += 2; // Skip both tiles
                        } else {
                            merged_line.push(line[i]);
                            i += 1;
                        }
                    }
                    line = merged_line;
                    
                    // Remplir avec des zéros et inverser
                    line.resize(GRID_SIZE, 0);
                    line.reverse();
                    
                    // Vérifier si quelque chose a changé
                    let new_row: [u32; GRID_SIZE] = line.as_slice().try_into().unwrap();
                    if new_grid[row] != new_row {
                        self.moved = true;
                    }
                    
                    new_grid[row] = new_row;
                }
            }
            Direction::Up => {
                for col in 0..GRID_SIZE {
                    let mut line: Vec<u32> = (0..GRID_SIZE)
                        .map(|row| new_grid[row][col])
                        .filter(|&x| x != 0)
                        .collect();
                    
                    // Fusionner les tuiles adjacentes identiques
                    let mut merged_line = Vec::new();
                    let mut i = 0;
                    while i < line.len() {
                        if i + 1 < line.len() && line[i] == line[i + 1] {
                            let merged_value = line[i] * 2;
                            merged_line.push(merged_value);
                            self.score += merged_value;
                            if merged_value == 2048 && !self.won {
                                self.won = true;
                            }
                            i += 2; // Skip both tiles
                        } else {
                            merged_line.push(line[i]);
                            i += 1;
                        }
                    }
                    line = merged_line;
                    
                    // Remplir avec des zéros
                    line.resize(GRID_SIZE, 0);
                    
                    // Vérifier si quelque chose a changé et appliquer
                    for row in 0..GRID_SIZE {
                        if new_grid[row][col] != line[row] {
                            self.moved = true;
                        }
                        new_grid[row][col] = line[row];
                    }
                }
            }
            Direction::Down => {
                for col in 0..GRID_SIZE {
                    let mut line: Vec<u32> = (0..GRID_SIZE)
                        .map(|row| new_grid[row][col])
                        .filter(|&x| x != 0)
                        .collect();
                    line.reverse();
                    
                    // Fusionner les tuiles adjacentes identiques
                    let mut merged_line = Vec::new();
                    let mut i = 0;
                    while i < line.len() {
                        if i + 1 < line.len() && line[i] == line[i + 1] {
                            let merged_value = line[i] * 2;
                            merged_line.push(merged_value);
                            self.score += merged_value;
                            if merged_value == 2048 && !self.won {
                                self.won = true;
                            }
                            i += 2; // Skip both tiles
                        } else {
                            merged_line.push(line[i]);
                            i += 1;
                        }
                    }
                    line = merged_line;
                    
                    // Remplir avec des zéros et inverser
                    line.resize(GRID_SIZE, 0);
                    line.reverse();
                    
                    // Vérifier si quelque chose a changé et appliquer
                    for row in 0..GRID_SIZE {
                        if new_grid[row][col] != line[row] {
                            self.moved = true;
                        }
                        new_grid[row][col] = line[row];
                    }
                }
            }
        }
        
        self.grid = new_grid;
        
        // Ajouter une nouvelle tuile si quelque chose a bougé
        if self.moved {
            self.add_random_tile();
            
            // Vérifier la fin de jeu
            if !self.can_move() {
                self.game_over = true;
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
        
        self.add_random_tile();
        self.add_random_tile();
    }
    
    fn get_tile_color(value: u32) -> Color {
        match value {
            0 => Color::Rgb(205, 193, 180),
            2 => Color::Rgb(238, 228, 218),
            4 => Color::Rgb(237, 224, 200),
            8 => Color::Rgb(242, 177, 121),
            16 => Color::Rgb(245, 149, 99),
            32 => Color::Rgb(246, 124, 95),
            64 => Color::Rgb(246, 94, 59),
            128 => Color::Rgb(237, 207, 114),
            256 => Color::Rgb(237, 204, 97),
            512 => Color::Rgb(237, 200, 80),
            1024 => Color::Rgb(237, 197, 63),
            2048 => Color::Rgb(237, 194, 46),
            _ => Color::Rgb(60, 58, 50),
        }
    }
    
    fn get_text_color(value: u32) -> Color {
        match value {
            0..=4 => Color::Rgb(119, 110, 101),
            _ => Color::Rgb(249, 246, 242),
        }
    }
}

impl Game for Game2048 {
    fn name(&self) -> &'static str {
        "2048"
    }
    
    fn description(&self) -> &'static str {
        "Slide numbered tiles to combine them and reach 2048!"
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        if self.game_over || self.won {
            match key.code {
                KeyCode::Char('r') => {
                    self.restart();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('w') => {
                    self.move_tiles(Direction::Up);
                    GameAction::Continue
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.move_tiles(Direction::Down);
                    GameAction::Continue
                }
                KeyCode::Left | KeyCode::Char('a') => {
                    self.move_tiles(Direction::Left);
                    GameAction::Continue
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.move_tiles(Direction::Right);
                    GameAction::Continue
                }
                KeyCode::Char('r') => {
                    self.restart();
                    GameAction::Continue
                }
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            }
        }
    }
    
    fn update(&mut self) -> GameAction {
        GameAction::Continue
    }
    
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_2048_game(frame, self);
    }
    
    fn tick_rate(&self) -> Duration {
        Duration::from_millis(100) // Pas besoin d'être très rapide pour 2048
    }
}

fn draw_2048_game(frame: &mut ratatui::Frame, game: &Game2048) {
    let area = frame.area();
    
    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec score
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer avec instructions
    ]).split(area);
    
    // Fond sombre élégant
    let background = Block::new()
        .style(Style::default().bg(Color::Rgb(15, 20, 25)));
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
                .style(Style::default().bg(Color::Rgb(25, 35, 45)))
        );
    frame.render_widget(header, chunks[0]);
    
    // === ZONE DE JEU ===
    let game_area = chunks[1];
    let game_block = Block::bordered()
        .title(" Playing Field ".green().bold())
        .border_style(Style::new().green())
        .style(Style::default().bg(Color::Rgb(10, 15, 20)));
    frame.render_widget(game_block, game_area);
    
    let inner_area = game_area.inner(Margin { vertical: 1, horizontal: 2 });
    
    // Calculer les dimensions pour centrer la grille
    let cell_width = 8;  // Largeur de chaque cellule
    let cell_height = 3; // Hauteur de chaque cellule
    let grid_width = (GRID_SIZE as u16 * cell_width) + (GRID_SIZE as u16 - 1); // +espaces entre cellules
    let grid_height = (GRID_SIZE as u16 * cell_height) + (GRID_SIZE as u16 - 1);
    
    let start_x = inner_area.x + (inner_area.width.saturating_sub(grid_width)) / 2;
    let start_y = inner_area.y + (inner_area.height.saturating_sub(grid_height)) / 2;
    
    // Dessiner la grille
    for row in 0..GRID_SIZE {
        for col in 0..GRID_SIZE {
            let value = game.grid[row][col];
            
            let cell_x = start_x + (col as u16 * (cell_width + 1));
            let cell_y = start_y + (row as u16 * (cell_height + 1));
            
            let cell_area = Rect {
                x: cell_x,
                y: cell_y,
                width: cell_width,
                height: cell_height,
            };
            
            let cell_text = if value == 0 {
                String::new()
            } else {
                format!("{}", value)
            };
            
            let cell_color = Game2048::get_tile_color(value);
            let text_color = Game2048::get_text_color(value);
            
            let cell = Paragraph::new(cell_text)
                .alignment(ratatui::layout::Alignment::Center)
                .block(
                    Block::bordered()
                        .style(Style::default().bg(cell_color))
                        .border_style(Style::default().fg(Color::Rgb(187, 173, 160)))
                )
                .style(Style::default().fg(text_color).bold());
            
            frame.render_widget(cell, cell_area);
        }
    }
    
    // === FOOTER ===
    let instructions = if game.game_over || game.won {
        vec![
            Line::from(vec![
                if game.won { "🎉 YOU WON! 🎉".green().bold() } else { "GAME OVER".red().bold() },
                "  ".white(),
                "R".green().bold(),
                " Restart  ".white(),
                "Q".red().bold(),
                " Quit".white(),
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
        ]
    };
    
    let footer = Paragraph::new(instructions)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45)))
        );
    frame.render_widget(footer, chunks[2]);
    
    // === POPUP DE VICTOIRE ===
    if game.won && !game.game_over {
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
                    .style(Style::default().bg(Color::Rgb(0, 50, 0)))
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