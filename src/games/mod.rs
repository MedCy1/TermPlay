use crate::core::{Game, GameInfo};
use std::collections::HashMap;

pub mod snake;

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
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}