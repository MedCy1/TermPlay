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
}

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
        }
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
            if (x.abs_diff(first_click_x) <= 1 && y.abs_diff(first_click_y) <= 1) || self.grid[y][x].is_mine {
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

        let cell = &self.grid[y][x];

        if cell.is_mine {
            self.game_over = true;
            // Révéler toutes les mines
            for row in &mut self.grid {
                for cell in row {
                    if cell.is_mine {
                        cell.state = CellState::Revealed;
                    }
                }
            }
            return;
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
                        self.reveal_cell(nx as usize, ny as usize);
                    }
                }
            }
        }

        // Vérifier la victoire
        if self.cells_revealed == (GRID_WIDTH * GRID_HEIGHT - MINE_COUNT) {
            self.won = true;
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
                }
            }
            CellState::Flagged => {
                cell.state = CellState::Hidden;
                self.flags_used -= 1;
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
    fn name(&self) -> &'static str {
        "Minesweeper"
    }

    fn description(&self) -> &'static str {
        "Classic mine detection game"
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
        draw_minesweeper_game(frame, self);
    }

    fn tick_rate(&self) -> Duration {
        Duration::from_millis(100)
    }
}

fn draw_minesweeper_game(frame: &mut ratatui::Frame, game: &MinesweeperGame) {
    let area = frame.area();

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec infos
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer avec instructions
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
            format!("{}", mines_left).white().bold(),
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

    // Dessiner la grille
    for row in 0..GRID_HEIGHT {
        for col in 0..GRID_WIDTH {
            let cell = &game.grid[row][col];

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

    // === FOOTER ===
    let instructions = if game.game_over || game.won {
        vec![Line::from(vec![
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
        ])]
    } else {
        vec![Line::from(vec![
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
        ])]
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