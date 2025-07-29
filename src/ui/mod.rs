use crate::core::{GameAction, GameInfo};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct MainMenu {
    games: Vec<GameInfo>,
    list_state: ListState,
}

impl MainMenu {
    pub fn new(games: Vec<&GameInfo>) -> Self {
        let mut list_state = ListState::default();
        if !games.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            games: games.into_iter().cloned().collect(),
            list_state,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match key.code {
            KeyCode::Char('q') => GameAction::Quit,
            KeyCode::Down => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i >= self.games.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                GameAction::Continue
            }
            KeyCode::Up => {
                let i = match self.list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.games.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.list_state.select(Some(i));
                GameAction::Continue
            }
            KeyCode::Enter => GameAction::GameOver,
            _ => GameAction::Continue,
        }
    }

    pub fn get_selected_game(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.games.get(i))
            .map(|game| game.name.as_str())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(size);

        let title = Paragraph::new("🎮 TermPlay - Terminal Mini-Games")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = self
            .games
            .iter()
            .map(|game| {
                let content = vec![Line::from(Span::styled(
                    format!("{} - {}", game.name, game.description),
                    Style::default(),
                ))];
                ListItem::new(content)
            })
            .collect();

        let games_list = List::new(items)
            .block(Block::default().title("Select a game").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(games_list, chunks[1], &mut self.list_state);

        let instructions = Paragraph::new("↑/↓: Navigate | Enter: Select | q: Quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(instructions, chunks[2]);
    }
}