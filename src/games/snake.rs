use crate::core::{Game, GameAction};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
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
        let width = 40;
        let height = 20;
        let snake = vec![Position { x: 10, y: 10 }];
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
                x: rng.random_range(1..width - 1),
                y: rng.random_range(1..height - 1),
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

        if new_head.x == 0
            || new_head.x >= self.width - 1
            || new_head.y == 0
            || new_head.y >= self.height - 1
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

    fn render(&self, frame: &mut Frame) {
        let size = frame.area();

        // Fond coloré pour toute l'interface
        let background = Block::default()
            .style(Style::default().bg(Color::Rgb(20, 30, 20)));
        frame.render_widget(background, size);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .split(size);

        // Interface de score stylée
        let score_text = vec![
            Line::from(vec![
                Span::styled("🐍 SNAKE GAME 🐍", 
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Score: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}", self.score), 
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::styled("   |   ", Style::default().fg(Color::Gray)),
                Span::styled("⬆️⬇️⬅️➡️", Style::default().fg(Color::Cyan)),
                Span::styled(" Move   ", Style::default().fg(Color::White)),
                Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" Quit", Style::default().fg(Color::White)),
            ]),
        ];

        let score_paragraph = Paragraph::new(score_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Rgb(15, 25, 15)))
            );
        frame.render_widget(score_paragraph, chunks[0]);

        let game_area = chunks[1];
        let game_block = Block::default()
            .title(vec![
                Span::styled("┤ ", Style::default().fg(Color::Green)),
                Span::styled("GAME FIELD", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" ├", Style::default().fg(Color::Green)),
            ])
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .style(Style::default().bg(Color::Rgb(10, 20, 10)));
        frame.render_widget(game_block, game_area);

        let inner_area = game_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let cell_width = inner_area.width / self.width;
        let cell_height = inner_area.height / self.height;

        // Dessiner le serpent avec des couleurs dégradées
        for (i, segment) in self.snake.iter().enumerate() {
            let x = inner_area.x + segment.x * cell_width;
            let y = inner_area.y + segment.y * cell_height;
            let cell_area = ratatui::layout::Rect {
                x,
                y,
                width: cell_width,
                height: cell_height,
            };
            
            // Couleur dégradée pour le serpent (tête plus claire)
            let color = if i == 0 {
                Color::Rgb(100, 255, 100) // Tête verte claire
            } else {
                Color::Rgb(50, 200, 50) // Corps vert plus foncé
            };
            
            let snake_cell = Block::default()
                .style(Style::default().bg(color));
            frame.render_widget(snake_cell, cell_area);
        }

        // Nourriture avec animation visuelle
        let food_x = inner_area.x + self.food.x * cell_width;
        let food_y = inner_area.y + self.food.y * cell_height;
        let food_area = ratatui::layout::Rect {
            x: food_x,
            y: food_y,
            width: cell_width,
            height: cell_height,
        };
        
        let food_cell = Block::default()
            .style(Style::default().bg(Color::Rgb(255, 100, 100))); // Rouge vif
        frame.render_widget(food_cell, food_area);

        // Game Over stylé
        if self.game_over {
            let popup_area = ratatui::layout::Rect {
                x: size.width / 4,
                y: size.height / 2 - 4,
                width: size.width / 2,
                height: 8,
            };

            let game_over_text = vec![
                Line::from(vec![
                    Span::styled("╔════════════════════════╗", 
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled("      💀 GAME OVER 💀     ", 
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("   Final Score: {:<8} ", self.score), 
                        Style::default().fg(Color::White)),
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled("                        ", Style::default()),
                    Span::styled("║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("║ ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled("R", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(" Restart  ", Style::default().fg(Color::White)),
                    Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::styled(" Quit ", Style::default().fg(Color::White)),
                    Span::styled(" ║", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::styled("╚════════════════════════╝", 
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                ]),
            ];

            let game_over_paragraph = Paragraph::new(game_over_text)
                .alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black));

            frame.render_widget(Clear, popup_area);
            frame.render_widget(game_over_paragraph, popup_area);
        }
    }
}