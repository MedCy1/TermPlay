use crossterm::event::KeyEvent;
use ratatui::Frame;
use std::error::Error;

pub type GameResult = Result<(), Box<dyn Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum GameAction {
    Continue,
    Quit,
    GameOver,
}

pub trait Game {
    fn handle_key(&mut self, key: KeyEvent) -> GameAction;
    fn update(&mut self) -> GameAction;
    fn draw(&mut self, frame: &mut Frame);
    fn tick_rate(&self) -> std::time::Duration {
        std::time::Duration::from_millis(250) // Valeur par défaut
    }
    /// Opt-in: intervalle de rendu (ex: 16ms) pour les animations, indépendant du tick logique.
    fn frame_time(&self) -> Option<std::time::Duration> {
        None
    }
    /// Appelé à chaque frame si `frame_time` est Some; `dt` = temps réel écoulé.
    fn animate(&mut self, _dt: std::time::Duration) {}
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
