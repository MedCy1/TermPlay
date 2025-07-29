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

        // Créer un dégradé de fond
        let background = Block::default()
            .style(Style::default().bg(Color::Rgb(25, 25, 35)));
        frame.render_widget(background, size);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),  // Zone titre plus grande
                Constraint::Min(0),     // Zone jeux
                Constraint::Length(4),  // Zone instructions
            ])
            .split(size);

        // Titre avec style ASCII art
        let title_text = vec![
            Line::from(vec![
                Span::styled("╔═══════════════════════════════════════════════════════════════╗", 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("║", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("                     🎮 TERMPLAY 🎮                          ", 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("║", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("║", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled("              Terminal Mini-Games Collection                  ", 
                    Style::default().fg(Color::Magenta)),
                Span::styled("║", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("╚═══════════════════════════════════════════════════════════════╝", 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
        ];

        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Rgb(15, 15, 25)));
        frame.render_widget(title, chunks[0]);

        // Liste des jeux avec style amélioré
        let items: Vec<ListItem> = self
            .games
            .iter()
            .enumerate()
            .map(|(_i, game)| {
                let icon = match game.name.as_str() {
                    "snake" => "🐍",
                    "tetris" => "🧩",
                    "pong" => "🏓",
                    _ => "🎮",
                };
                
                let content = vec![Line::from(vec![
                    Span::styled(format!("  {} ", icon), 
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<12}", game.name.to_uppercase()), 
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(" │ ", Style::default().fg(Color::Gray)),
                    Span::styled(&game.description, 
                        Style::default().fg(Color::LightBlue)),
                ])];
                ListItem::new(content)
            })
            .collect();

        let games_list = List::new(items)
            .block(
                Block::default()
                    .title(vec![
                        Span::styled("┤ ", Style::default().fg(Color::Cyan)),
                        Span::styled("SELECT GAME", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Span::styled(" ├", Style::default().fg(Color::Cyan)),
                    ])
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .style(Style::default().bg(Color::Rgb(15, 15, 25)))
            )
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(0, 150, 200))
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(games_list, chunks[1], &mut self.list_state);

        // Instructions avec style amélioré
        let instructions_text = vec![
            Line::from(vec![
                Span::styled("⬆️⬇️", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" Navigate  ", Style::default().fg(Color::White)),
                Span::styled("⏎", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" Select  ", Style::default().fg(Color::White)),
                Span::styled("Q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled(" Quit", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to start your adventure!", Style::default().fg(Color::Gray)),
            ]),
        ];

        let instructions = Paragraph::new(instructions_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Rgb(15, 15, 25)))
            );
        frame.render_widget(instructions, chunks[2]);
    }
}