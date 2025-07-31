use crate::core::{Game, GameAction, GameResult};
use crate::games::GameRegistry;
use crate::menu::MainMenu;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

pub struct App {
    registry: GameRegistry,
}

impl App {
    pub fn new() -> Self {
        Self {
            registry: GameRegistry::new(),
        }
    }

    pub fn run_game(&mut self, game_name: &str) -> GameResult {
        if let Some(mut game) = self.registry.get_game(game_name) {
            let mut terminal = self.setup_terminal()?;
            let result = self.run_game_loop(&mut game, &mut terminal);
            self.restore_terminal(&mut terminal)?;
            result
        } else {
            eprintln!("Game '{game_name}' not found!");
            Ok(())
        }
    }

    pub fn run_menu(&mut self) -> GameResult {
        let mut terminal = self.setup_terminal()?;
        let mut menu = MainMenu::new(self.registry.list_games())
            .map_err(|e| format!("Failed to initialize menu: {e}"))?;
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| menu.draw(f))?;

            let timeout = Duration::from_millis(100)
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Ne traiter que les événements de pression de touche pour éviter les répétitions
                    if key.kind == KeyEventKind::Press {
                        match menu.handle_key(key) {
                            GameAction::Quit => break,
                            GameAction::Continue => continue,
                            GameAction::GameOver => {
                                if let Some(selected_game) = menu.get_selected_game() {
                                    if let Some(mut game) = self.registry.get_game(selected_game) {
                                        self.run_game_loop(&mut game, &mut terminal)?;
                                        // Recréer le menu après le jeu en préservant la configuration
                                        menu = MainMenu::new(self.registry.list_games()).map_err(
                                            |e| format!("Failed to recreate menu: {e}"),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Update du menu pour gérer la musique
            if last_tick.elapsed() >= Duration::from_millis(100) {
                menu.update();
                last_tick = Instant::now();
            }
        }

        self.restore_terminal(&mut terminal)?;
        Ok(())
    }

    pub fn list_games(&self) {
        println!("Available games:");
        for game_info in self.registry.list_games() {
            println!("  {} - {}", game_info.name, game_info.description);
        }
    }

    pub fn has_game(&self, name: &str) -> bool {
        self.registry.has_game(name)
    }

    fn setup_terminal(
        &self,
    ) -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        Ok(Terminal::new(backend)?)
    }

    fn restore_terminal(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> GameResult {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn run_game_loop<B: Backend>(
        &self,
        game: &mut Box<dyn Game>,
        terminal: &mut Terminal<B>,
    ) -> GameResult {
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| game.draw(f))?;

            let tick_rate = game.tick_rate(); // Obtenir le tick rate dynamique
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // Ne traiter que les événements de pression de touche
                    if key.kind == KeyEventKind::Press {
                        match game.handle_key(key) {
                            GameAction::Quit => break,
                            GameAction::GameOver => break,
                            GameAction::Continue => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                match game.update() {
                    GameAction::Quit => break,
                    GameAction::GameOver => break,
                    GameAction::Continue => {}
                }
                last_tick = Instant::now();
            }
        }

        Ok(())
    }
}
