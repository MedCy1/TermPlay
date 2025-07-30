use crate::core::{Game, GameAction};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph},
};

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
}

impl SnakeGame {
    pub fn new() -> Self {
        // Dimensions par défaut, seront mises à jour lors du premier rendu
        let width = 40;
        let height = 20;
        let snake = vec![Position { x: width / 2, y: height / 2 }];
        let food = Self::generate_food(&snake, width, height);

        Self {
            snake,
            direction: SnakeDirection::Right,
            food,
            score: 0,
            game_over: false,
            width,
            height,
        }
    }


    fn generate_food(snake: &[Position], width: u16, height: u16) -> Position {
        let mut rng = rand::rng();
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

        if new_head.x >= self.width
            || new_head.y >= self.height
            || self.snake.contains(&new_head)
        {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);

        if new_head == self.food {
            self.score += 10;
            self.food = Self::generate_food(&self.snake, self.width, self.height);
        } else {
            self.snake.pop();
        }
    }
}


impl Game for SnakeGame {
    fn name(&self) -> &str {
        "snake"
    }

    fn description(&self) -> &str {
        "Classic Snake game"
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
                _ => GameAction::Continue,
            }
        }
    }

    fn update(&mut self) -> GameAction {
        if !self.game_over {
            self.move_snake();
        }
        GameAction::Continue
    }

    fn draw(&mut self, frame: &mut ratatui::Frame) {
        draw_snake_game(frame, self);
    }
}

impl SnakeGame {
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
}

fn draw_snake_game(frame: &mut ratatui::Frame, app: &mut SnakeGame) {
    let area = frame.area();
    
    // Layout principal d'abord pour connaître l'espace réel disponible
    let chunks = Layout::vertical([
        Constraint::Length(4), // Header avec score
        Constraint::Min(0),    // Zone de jeu
        Constraint::Length(3), // Footer avec instructions
    ]).split(area);

    let game_area = chunks[1];
    let inner_area = game_area.inner(Margin { vertical: 1, horizontal: 1 });
    
    // Calculer les VRAIES dimensions utilisables = exactement l'espace disponible
    let game_width = inner_area.width.max(20); // Pas de limite max
    let game_height = inner_area.height.max(10); // Pas de limite max
    
    // Mettre à jour les dimensions logiques du jeu
    app.update_dimensions(game_width, game_height);

    // Fond sombre élégant
    let background = Block::new()
        .style(Style::default().bg(Color::Rgb(15, 20, 25)));
    frame.render_widget(background, area);

    // === HEADER ===
    let header_text = vec![
        Line::from(vec![
            "🐍 ".green().bold(),
            "SNAKE GAME".cyan().bold(),
            " 🐍".green().bold(),
        ]),
        Line::from(vec![
            "Score: ".yellow(),
            format!("{}", app.score).white().bold(),
            format!(" | Game: {}x{} | Inner: {}x{}", game_width, game_height, inner_area.width, inner_area.height).gray(),
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

    let inner_area = game_area.inner(Margin { vertical: 1, horizontal: 1 });
    
    // Utiliser exactement 1 caractère par cellule de jeu
    // Pas d'offset car nous utilisons tout l'espace disponible
    let offset_x = 0;
    let offset_y = 0;

    // Dessiner le serpent avec style dégradé
    for (i, segment) in app.snake.iter().enumerate() {
        if segment.x < game_width && segment.y < game_height {
            let x = inner_area.x + offset_x + segment.x;
            let y = inner_area.y + offset_y + segment.y;
            
            let cell_area = Rect {
                x,
                y,
                width: 1,
                height: 1,
            };
            
            // Couleurs dégradées pour un effet visuel
            let (color, symbol) = if i == 0 {
                (Color::Rgb(120, 255, 120), "█") // Tête verte claire
            } else {
                let intensity = 180 - (i * 10).min(100) as u8;
                (Color::Rgb(50, intensity, 50), "█") // Corps dégradé
            };
            
            let snake_cell = Paragraph::new(symbol)
                .style(Style::default().fg(color));
            frame.render_widget(snake_cell, cell_area);
        }
    }

    // Dessiner la nourriture avec animation
    if app.food.x < game_width && app.food.y < game_height {
        let food_x = inner_area.x + offset_x + app.food.x;
        let food_y = inner_area.y + offset_y + app.food.y;
        
        let food_area = Rect {
            x: food_x,
            y: food_y,
            width: 1,
            height: 1,
        };
        
        let food_cell = Paragraph::new("●")
            .style(Style::default().fg(Color::Red).bold());
        frame.render_widget(food_cell, food_area);
    }

    // === FOOTER ===
    let instructions = vec![
        Line::from(vec![
            "Arrow Keys".cyan().bold(),
            " Move  ".white(),
            "Q".red().bold(),
            " Quit  ".white(),
            if app.game_over { "R".green().bold() } else { "".white() },
            if app.game_over { " Restart" } else { "" }.white(),
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
    if app.game_over {
        let popup_width = 40.min(area.width);
        let popup_height = 8.min(area.height);
        let popup_area = Rect {
            x: if area.width >= popup_width { (area.width - popup_width) / 2 } else { 0 },
            y: if area.height >= popup_height { (area.height - popup_height) / 2 } else { 0 },
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
                    .style(Style::default().bg(Color::Black))
            );
        frame.render_widget(popup, popup_area);
    }
}