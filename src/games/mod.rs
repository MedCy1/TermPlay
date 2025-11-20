use crate::core::{Game, GameInfo};
use std::collections::HashMap;

pub mod _2048;
pub mod breakout;
pub mod gameoflife;
pub mod minesweeper;
pub mod pong;
pub mod snake;
pub mod tetris;

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
        self.games.insert(name.to_string(), Box::new(constructor));
        self.info
            .insert(name.to_string(), GameInfo::new(name, description));
    }

    pub fn get_game(&self, name: &str) -> Option<Box<dyn Game>> {
        self.games.get(name).map(|constructor| constructor())
    }

    pub fn list_games(&self) -> Vec<&GameInfo> {
        let mut games: Vec<&GameInfo> = self.info.values().collect();
        games.sort_by(|a, b| a.name.cmp(&b.name));
        games
    }

    pub fn has_game(&self, name: &str) -> bool {
        self.games.contains_key(name)
    }

    fn register_all_games(&mut self) {
        // Enregistrer les jeux avec des métadonnées statiques pour éviter l'initialisation audio
        self.register("snake", "Classic Snake game", || {
            Box::new(snake::SnakeGame::new())
        });

        self.register("tetris", "Classic Tetris with line clearing", || {
            Box::new(tetris::TetrisGame::new())
        });

        self.register("pong", "Classic Pong with 1 or 2 players", || {
            Box::new(pong::PongGame::new())
        });

        self.register(
            "2048",
            "Slide numbered tiles to combine them and reach 2048!",
            || Box::new(_2048::Game2048::new()),
        );

        self.register("Minesweeper", "Classic mine detection game", || {
            Box::new(minesweeper::MinesweeperGame::new())
        });

        self.register("Breakout", "Brick breaking arcade game", || {
            Box::new(breakout::BreakoutGame::new())
        });

        self.register(
            "Game of Life",
            "Conway's Game of Life - Cellular automaton visualization",
            || Box::new(gameoflife::GameOfLife::new()),
        );
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}
