use crate::core::{Game, GameAction};
use crossterm::event::{KeyCode, KeyEvent};
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(size);

        let score_text = format!("Score: {} | Controls: ↑↓←→ | q: Quit", self.score);
        let score_paragraph = Paragraph::new(score_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(score_paragraph, chunks[0]);

        let game_area = chunks[1];
        let game_block = Block::default()
            .title("🐍 Snake")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(game_block, game_area);

        let inner_area = game_area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        let cell_width = inner_area.width / self.width;
        let cell_height = inner_area.height / self.height;

        for segment in &self.snake {
            let x = inner_area.x + segment.x * cell_width;
            let y = inner_area.y + segment.y * cell_height;
            let cell_area = ratatui::layout::Rect {
                x,
                y,
                width: cell_width,
                height: cell_height,
            };
            
            let snake_cell = Block::default()
                .style(Style::default().bg(Color::Green));
            frame.render_widget(snake_cell, cell_area);
        }

        let food_x = inner_area.x + self.food.x * cell_width;
        let food_y = inner_area.y + self.food.y * cell_height;
        let food_area = ratatui::layout::Rect {
            x: food_x,
            y: food_y,
            width: cell_width,
            height: cell_height,
        };
        
        let food_cell = Block::default()
            .style(Style::default().bg(Color::Red));
        frame.render_widget(food_cell, food_area);

        if self.game_over {
            let popup_area = ratatui::layout::Rect {
                x: size.width / 4,
                y: size.height / 2 - 2,
                width: size.width / 2,
                height: 4,
            };

            let game_over_text = vec![
                Line::from("Game Over!"),
                Line::from(format!("Final Score: {}", self.score)),
                Line::from("Press 'r' to restart or 'q' to quit"),
            ];

            let game_over_paragraph = Paragraph::new(game_over_text)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black)),
                );

            frame.render_widget(ratatui::widgets::Clear, popup_area);
            frame.render_widget(game_over_paragraph, popup_area);
        }
    }
}