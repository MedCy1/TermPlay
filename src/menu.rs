use crate::core::{GameAction, GameInfo};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    Main,
    Games,
    Settings,
    About,
}

#[derive(Debug, Clone)]
pub struct MenuOption {
    pub title: String,
    pub description: String,
    pub action: MenuAction,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    EnterSubMenu(MenuState),
    LaunchGame(String),
    ShowAbout,
    Quit,
    GoBack,
}

pub struct MainMenu {
    current_menu: MenuState,
    main_options: Vec<MenuOption>,
    games_list: Vec<GameInfo>,
    selected_index: usize,
    list_state: ListState,
}

impl MainMenu {
    pub fn new(games: Vec<&GameInfo>) -> Self {
        let main_options = vec![
            MenuOption {
                title: "🎮 Games".to_string(),
                description: "Play exciting terminal games".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::Games),
            },
            MenuOption {
                title: "⚙️ Settings".to_string(),
                description: "Configure game preferences".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::Settings),
            },
            MenuOption {
                title: "ℹ️ About".to_string(),
                description: "About TermPlay".to_string(),
                action: MenuAction::EnterSubMenu(MenuState::About),
            },
            MenuOption {
                title: "🚪 Quit".to_string(),
                description: "Exit TermPlay".to_string(),
                action: MenuAction::Quit,
            },
        ];

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            current_menu: MenuState::Main,
            main_options,
            games_list: games.into_iter().cloned().collect(),
            selected_index: 0,
            list_state,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> GameAction {
        match key.code {
            KeyCode::Char('q') => {
                if self.current_menu == MenuState::Main {
                    GameAction::Quit
                } else {
                    self.go_back();
                    GameAction::Continue
                }
            }
            KeyCode::Esc => {
                if self.current_menu != MenuState::Main {
                    self.go_back();
                }
                GameAction::Continue
            }
            KeyCode::Down => {
                self.next_item();
                GameAction::Continue
            }
            KeyCode::Up => {
                self.previous_item();
                GameAction::Continue
            }
            KeyCode::Enter => self.select_current_item(),
            _ => GameAction::Continue,
        }
    }

    fn next_item(&mut self) {
        let max_items = match self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::Settings => 3, // Placeholder
            MenuState::About => 1,
        };

        self.selected_index = (self.selected_index + 1) % max_items;
        self.list_state.select(Some(self.selected_index));
    }

    fn previous_item(&mut self) {
        let max_items = match self.current_menu {
            MenuState::Main => self.main_options.len(),
            MenuState::Games => self.games_list.len(),
            MenuState::Settings => 3,
            MenuState::About => 1,
        };

        self.selected_index = if self.selected_index == 0 {
            max_items - 1
        } else {
            self.selected_index - 1
        };
        self.list_state.select(Some(self.selected_index));
    }

    fn select_current_item(&mut self) -> GameAction {
        match self.current_menu {
            MenuState::Main => {
                if let Some(option) = self.main_options.get(self.selected_index) {
                    match &option.action {
                        MenuAction::EnterSubMenu(menu_state) => {
                            self.current_menu = menu_state.clone();
                            self.selected_index = 0;
                            self.list_state.select(Some(0));
                            GameAction::Continue
                        }
                        MenuAction::Quit => GameAction::Quit,
                        _ => GameAction::Continue,
                    }
                } else {
                    GameAction::Continue
                }
            }
            MenuState::Games => {
                if let Some(_game) = self.games_list.get(self.selected_index) {
                    // Retourner GameOver avec le nom du jeu pour le lancer
                    GameAction::GameOver // On utilisera GameOver comme signal pour lancer le jeu
                } else {
                    GameAction::Continue
                }
            }
            MenuState::Settings | MenuState::About => {
                // Pour l'instant, juste revenir au menu principal
                self.go_back();
                GameAction::Continue
            }
        }
    }

    fn go_back(&mut self) {
        self.current_menu = MenuState::Main;
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    pub fn get_selected_game(&self) -> Option<&str> {
        if self.current_menu == MenuState::Games {
            self.games_list.get(self.selected_index).map(|g| g.name.as_str())
        } else {
            None
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        draw_main_menu(frame, self);
    }
}

fn draw_main_menu(frame: &mut Frame, menu: &MainMenu) {
    let area = frame.area();

    // Fond dégradé élégant
    let background = Block::new()
        .style(Style::default().bg(Color::Rgb(10, 15, 25)));
    frame.render_widget(background, area);

    // Layout principal
    let chunks = Layout::vertical([
        Constraint::Length(8),  // Header
        Constraint::Min(0),     // Contenu principal
        Constraint::Length(4),  // Footer
    ]).split(area);

    // === HEADER ===
    draw_header(frame, chunks[0], &menu.current_menu);

    // === CONTENU PRINCIPAL ===
    match menu.current_menu {
        MenuState::Main => draw_main_options(frame, chunks[1], menu),
        MenuState::Games => draw_games_menu(frame, chunks[1], menu),
        MenuState::Settings => draw_settings_menu(frame, chunks[1]),
        MenuState::About => draw_about_menu(frame, chunks[1]),
    }

    // === FOOTER ===
    draw_footer(frame, chunks[2], &menu.current_menu);
}

fn draw_header(frame: &mut Frame, area: Rect, current_menu: &MenuState) {
    let title = match current_menu {
        MenuState::Main => "TERMPLAY",
        MenuState::Games => "GAMES",
        MenuState::Settings => "SETTINGS", 
        MenuState::About => "ABOUT",
    };

    let subtitle = match current_menu {
        MenuState::Main => "Terminal Mini-Games Collection",
        MenuState::Games => "Choose your adventure",
        MenuState::Settings => "Configure your experience",
        MenuState::About => "Information about TermPlay",
    };

    let header_text = vec![
        Line::from(""),
        Line::from(vec![
            "╔════════════════════════════════════════════════════════════════╗".cyan().bold(),
        ]),
        Line::from(vec![
            "║".cyan().bold(),
            format!("                     🎮 {} 🎮                     ", title).yellow().bold(),
            "║".cyan().bold(),
        ]),
        Line::from(vec![
            "║".cyan().bold(),
            format!("           {}            ", subtitle).magenta(),
            "║".cyan().bold(),
        ]),
        Line::from(vec![
            "╚════════════════════════════════════════════════════════════════╝".cyan().bold(),
        ]),
        Line::from(""),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(15, 25, 35)));
    frame.render_widget(header, area);
}

fn draw_main_options(frame: &mut Frame, area: Rect, menu: &MainMenu) {
    let items: Vec<ListItem> = menu
        .main_options
        .iter()
        .map(|option| {
            let content = vec![Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(&option.title, Style::default().fg(Color::White).bold()),
                Span::styled("  -  ", Style::default().fg(Color::Gray)),
                Span::styled(&option.description, Style::default().fg(Color::LightBlue)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Main Menu ".white().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(15, 20, 30)))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 100, 200))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut menu.list_state.clone());
}

fn draw_games_menu(frame: &mut Frame, area: Rect, menu: &MainMenu) {
    let items: Vec<ListItem> = menu
        .games_list
        .iter()
        .map(|game| {
            let icon = match game.name.as_str() {
                "snake" => "🐍",
                "tetris" => "🧩",
                "pong" => "🏓",
                _ => "🎮",
            };

            let content = vec![Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(Color::Green).bold()),
                Span::styled(game.name.to_uppercase(), Style::default().fg(Color::White).bold()),
                Span::styled("  -  ", Style::default().fg(Color::Gray)),
                Span::styled(&game.description, Style::default().fg(Color::LightBlue)),
            ])];
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Available Games ".green().bold())
                .border_style(Style::new().green())
                .style(Style::default().bg(Color::Rgb(15, 25, 15)))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(0, 150, 50))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut menu.list_state.clone());
}

fn draw_settings_menu(frame: &mut Frame, area: Rect) {
    let settings_text = vec![
        Line::from(""),
        Line::from("⚙️ Settings Menu".yellow().bold()),
        Line::from(""),
        Line::from("🎨 Graphics Settings"),
        Line::from("🔊 Audio Settings"),
        Line::from("⌨️ Controls Settings"),
        Line::from(""),
        Line::from("(Coming soon...)".gray().italic()),
    ];

    let settings = Paragraph::new(settings_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Settings ".yellow().bold())
                .border_style(Style::new().yellow())
                .style(Style::default().bg(Color::Rgb(25, 20, 15)))
        );
    frame.render_widget(settings, area);
}

fn draw_about_menu(frame: &mut Frame, area: Rect) {
    let about_text = vec![
        Line::from(""),
        Line::from("🎮 TermPlay v0.1.0".cyan().bold()),
        Line::from(""),
        Line::from("A beautiful collection of terminal mini-games"),
        Line::from("built with Rust and Ratatui."),
        Line::from(""),
        Line::from("Features:".yellow().bold()),
        Line::from("• Classic game with modern graphics"),
        Line::from("• Responsive design that adapts to terminal size"),
        Line::from("• Extensible architecture for adding new games"),
        Line::from(""),
        Line::from("Created with ❤️ by MedCy1 using Rust".red()),
    ];

    let about = Paragraph::new(about_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" About TermPlay ".cyan().bold())
                .border_style(Style::new().cyan())
                .style(Style::default().bg(Color::Rgb(15, 20, 25)))
        );
    frame.render_widget(about, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, current_menu: &MenuState) {
    let controls = match current_menu {
        MenuState::Main => "↑/↓ Navigate • Enter Select • Q Quit",
        _ => "↑/↓ Navigate • Enter Select • Esc/Q Back",
    };

    let footer_text = vec![
        Line::from(vec![
            "Controls: ".gray(),
            controls.white().bold(),
        ]),
        Line::from("Press Enter to select, Esc to go back".gray()),
    ];

    let footer = Paragraph::new(footer_text)
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" Controls ".blue().bold())
                .border_style(Style::new().blue())
                .style(Style::default().bg(Color::Rgb(20, 25, 35)))
        );
    frame.render_widget(footer, area);
}