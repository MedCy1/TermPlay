use crossterm::event::KeyEvent;
use std::error::Error;

pub type GameResult = Result<(), Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    Continue,
    Quit,
    GameOver,
}

pub trait Game {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn handle_key(&mut self, key: KeyEvent) -> GameAction;
    fn update(&mut self) -> GameAction;
    fn render(&self, frame: &mut ratatui::Frame);
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub name: String,
    pub description: String,
}

impl GameInfo {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}