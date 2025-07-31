mod app;
mod audio;
mod cli;
mod config;
mod core;
mod games;
mod menu;
mod music;

use app::App;
use clap::Parser;
use cli::{Cli, Commands};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use std::io::{self, Write};

/// Fonction de nettoyage d'urgence du terminal
fn emergency_terminal_cleanup() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = io::stdout().flush();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Installer un hook de panic global pour nettoyer le terminal
    std::panic::set_hook(Box::new(|panic_info| {
        emergency_terminal_cleanup();
        eprintln!("Application panic: {panic_info}");
    }));
    let cli = Cli::parse();
    let mut app = App::new();

    match cli.command {
        Some(Commands::Game { name }) => {
            if app.has_game(&name) {
                app.run_game(&name)?;
            } else {
                eprintln!("Game '{name}' not found!");
                eprintln!("Use 'termplay list' to see available games.");
                std::process::exit(1);
            }
        }
        Some(Commands::List) => {
            app.list_games();
        }
        None => {
            app.run_menu()?;
        }
    }

    // Nettoyer le hook de panic à la sortie normale
    let _ = std::panic::take_hook();

    Ok(())
}
