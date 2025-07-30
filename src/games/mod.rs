use crate::core::{Game, GameInfo};
use std::collections::HashMap;

pub mod snake;
pub mod tetris;
pub mod pong;
pub mod game2048;
pub mod minesweeper;
pub mod breakout;
pub mod gameoflife;

pub type GameConstructor = Box<dyn Fn() -> Box<dyn Game>>;

pub struct GameRegistry {
    games: HashMap<String, GameConstructor>,
    info: HashMap<String, GameInfo>,
}

impl GameRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            games: HashMap::new(),
            info: HashMap::new(),
        };
        registry.register_all_games();
        registry
    }

    pub fn register<F>(&mut self, name: &str, description: &str, constructor: F)
    where
        F: Fn() -> Box<dyn Game> + 'static,
    {
        self.games
            .insert(name.to_string(), Box::new(constructor));
        self.info
            .insert(name.to_string(), GameInfo::new(name, description));
    }

    pub fn get_game(&self, name: &str) -> Option<Box<dyn Game>> {
        self.games.get(name).map(|constructor| constructor())
    }

    pub fn list_games(&self) -> Vec<&GameInfo> {
        self.info.values().collect()
    }

    pub fn has_game(&self, name: &str) -> bool {
        self.games.contains_key(name)
    }

    fn register_all_games(&mut self) {
        let snake_game = snake::SnakeGame::new();
        self.register(snake_game.name(), snake_game.description(), || {
            Box::new(snake::SnakeGame::new())
        });

        let tetris_game = tetris::TetrisGame::new();
        self.register(tetris_game.name(), tetris_game.description(), || {
            Box::new(tetris::TetrisGame::new())
        });

        let pong_game = pong::PongGame::new();
        self.register(pong_game.name(), pong_game.description(), || {
            Box::new(pong::PongGame::new())
        });

        let game2048 = game2048::Game2048::new();
        self.register(game2048.name(), game2048.description(), || {
            Box::new(game2048::Game2048::new())
        });

        let minesweeper_game = minesweeper::MinesweeperGame::new();
        self.register(minesweeper_game.name(), minesweeper_game.description(), || {
            Box::new(minesweeper::MinesweeperGame::new())
        });

        let breakout_game = breakout::BreakoutGame::new();
        self.register(breakout_game.name(), breakout_game.description(), || {
            Box::new(breakout::BreakoutGame::new())
        });

        let gameoflife = gameoflife::GameOfLife::new();
        self.register(gameoflife.name(), gameoflife.description(), || {
            Box::new(gameoflife::GameOfLife::new())
        });
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}