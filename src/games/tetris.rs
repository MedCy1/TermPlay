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
            PieceType::O => &[
                &[true, true],
                &[true, true],
            ],
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

    fn get_color(&self) -> Color {
        match self {
            PieceType::I => Color::Cyan,
            PieceType::O => Color::Yellow,
            PieceType::T => Color::Magenta,
            PieceType::S => Color::Green,
            PieceType::Z => Color::Red,
            PieceType::J => Color::Blue,
            PieceType::L => Color::Rgb(255, 165, 0), // Orange
        }
    }

    fn random() -> Self {
        let mut rng = rand::rng();
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

    fn get_blocks(&self) -> Vec<Position> {
        let shape = self.get_rotated_shape();
        let mut blocks = Vec::new();
        
        for (y, row) in shape.iter().enumerate() {
            for (x, &filled) in row.iter().enumerate() {
                if filled {
                    blocks.push(Position {
                        x: self.position.x + x as i32,
                        y: self.position.y + y as i32,
                    });
                }
            }
        }
        blocks
    }

    fn get_rotated_shape(&self) -> Vec<Vec<bool>> {
        let original = self.piece_type.get_shape();
        let mut shape: Vec<Vec<bool>> = original.iter().map(|row| row.to_vec()).collect();
        
        for _ in 0..self.rotation {
            shape = Self::rotate_shape(shape);
        }
        shape
    }

    fn rotate_shape(shape: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
        let rows = shape.len();
        let cols = shape[0].len();
        let mut rotated = vec![vec![false; rows]; cols];
        
        for i in 0..rows {
            for j in 0..cols {
                rotated[j][rows - 1 - i] = shape[i][j];
            }
        }
        rotated
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
        };
        game.spawn_piece();
        game
    }

    fn spawn_piece(&mut self) {
        let new_piece = Piece::new(self.next_piece);
        self.next_piece = PieceType::random();
        
        if self.is_valid_position(&new_piece) {
            self.current_piece = Some(new_piece);
        } else {
            self.game_over = true;
        }
    }

    fn is_valid_position(&self, piece: &Piece) -> bool {
        for block in piece.get_blocks() {
            if block.x < 0 || block.x >= BOARD_WIDTH as i32 || 
               block.y >= BOARD_HEIGHT as i32 ||
               (block.y >= 0 && self.board[block.y as usize][block.x as usize].is_some()) {
                return false;
            }
        }
        true
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
        self.clear_lines();
        self.spawn_piece();
    }

    fn clear_lines(&mut self) {
        let mut lines_to_clear = Vec::new();
        
        // Identifier les lignes complètes
        for y in 0..BOARD_HEIGHT {
            if self.board[y].iter().all(|cell| cell.is_some()) {
                lines_to_clear.push(y);
            }
        }

        // Supprimer les lignes complètes et les remplacer
        for &line in lines_to_clear.iter().rev() {
            for y in (1..=line).rev() {
                self.board[y] = self.board[y - 1];
            }
            self.board[0] = [None; BOARD_WIDTH];
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

    fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(piece) = &self.current_piece {
            let new_piece = piece.moved(dx, dy);
            if self.is_valid_position(&new_piece) {
                self.current_piece = Some(new_piece);
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
        while self.move_piece(0, 1) {
            self.score += 2; // Points bonus pour hard drop
        }
        self.place_piece();
    }

    fn get_drop_interval(&self) -> u32 {
        // Vitesse progressive basée sur le niveau
        std::cmp::max(1, 21 - self.level)
    }
}

impl Game for TetrisGame {
    fn name(&self) -> &str {
        "tetris"
    }

    fn description(&self) -> &str {
        "Classic Tetris with line clearing"
    }

    fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        if self.game_over {
            match key.code {
                KeyCode::Char('r') => {
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
                KeyCode::Char('q') => GameAction::Quit,
                _ => GameAction::Continue,
            }
        }
    }

    fn update(&mut self) -> GameAction {
        if !self.game_over {
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

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(50) // Plus rapide pour une meilleure réactivité
    }
}

fn draw_tetris_game(frame: &mut ratatui::Frame, game: &TetrisGame) {
    let area = frame.area();
    
    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer
    ]).split(area);

    // Fond sombre
    let background = Block::new()
        .style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let header_text = vec![
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
    let inner_area = game_area.inner(Margin { vertical: 1, horizontal: 2 });
    
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
        height: BOARD_HEIGHT as u16, // Forcer exactement la hauteur de la grille logique
    };

    // Dessiner la grille (exactement BOARD_HEIGHT lignes)
    for y in 0..BOARD_HEIGHT {
        for x in 0..BOARD_WIDTH {
            let cell_x = board_area.x + (x as u16 * 2);
            let cell_y = board_area.y + y as u16;
            
            if cell_x + 1 < board_area.x + board_area.width && y < BOARD_HEIGHT {
                let cell_area = Rect {
                    x: cell_x,
                    y: cell_y,
                    width: 2,
                    height: 1,
                };

                let (symbol, color) = if let Some(piece_type) = game.board[y][x] {
                    ("██", piece_type.get_color())
                } else {
                    ("░░", Color::Rgb(40, 40, 50))
                };

                let cell = Paragraph::new(symbol)
                    .style(Style::default().fg(color));
                frame.render_widget(cell, cell_area);
            }
        }
    }

    // Dessiner la pièce actuelle
    if let Some(piece) = &game.current_piece {
        for block in piece.get_blocks() {
            if block.x >= 0 && block.x < BOARD_WIDTH as i32 && 
               block.y >= 0 && block.y < BOARD_HEIGHT as i32 {
                let cell_x = board_area.x + (block.x as u16 * 2);
                let cell_y = board_area.y + block.y as u16;
                
                if cell_x + 1 < board_area.x + board_area.width {
                    let cell_area = Rect {
                        x: cell_x,
                        y: cell_y,
                        width: 2,
                        height: 1,
                    };

                    let cell = Paragraph::new("██")
                        .style(Style::default().fg(piece.piece_type.get_color()).bold());
                    frame.render_widget(cell, cell_area);
                }
            }
        }
    }

    // Dessiner les infos à côté (prochaine pièce)
    let info_area = Rect {
        x: board_area.x + board_width + 2,
        y: board_area.y,
        width: game_rect.width.saturating_sub(board_width + 3),
        height: 8,
    };

    if info_area.width > 0 {
        let next_text = vec![
            Line::from("Next:".yellow().bold()),
            Line::from(""),
        ];
        
        let next_info = Paragraph::new(next_text)
            .block(
                Block::bordered()
                    .title(" Next ".yellow())
                    .border_style(Style::new().yellow())
            );
        frame.render_widget(next_info, info_area);

        // Dessiner la prochaine pièce
        let next_shape = game.next_piece.get_shape();
        for (y, row) in next_shape.iter().enumerate() {
            for (x, &filled) in row.iter().enumerate() {
                if filled {
                    let piece_x = info_area.x + 2 + (x as u16 * 2);
                    let piece_y = info_area.y + 3 + y as u16;
                    
                    if piece_x + 1 < info_area.x + info_area.width &&
                       piece_y < info_area.y + info_area.height {
                        let piece_area = Rect {
                            x: piece_x,
                            y: piece_y,
                            width: 2,
                            height: 1,
                        };

                        let piece_cell = Paragraph::new("██")
                            .style(Style::default().fg(game.next_piece.get_color()));
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
            " Hard Drop  ".white(),
            "Q".red().bold(),
            " Quit".white(),
        ]),
    ];
    
    let footer = Paragraph::new(instructions)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".white().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(25, 35, 45)))
        );
    frame.render_widget(footer, chunks[2]);

    // === GAME OVER POPUP ===
    if game.game_over {
        let popup_width = 50.min(area.width);
        let popup_height = 10.min(area.height);
        let popup_area = Rect {
            x: if area.width >= popup_width { (area.width - popup_width) / 2 } else { 0 },
            y: if area.height >= popup_height { (area.height - popup_height) / 2 } else { 0 },
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
                    .style(Style::default().bg(Color::Black))
            );
        frame.render_widget(popup, popup_area);
    }
}