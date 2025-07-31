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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut app = App::new();

    match cli.command {
        Some(Commands::Game { name }) => {
            if app.has_game(&name) {
                app.run_game(&name)?;
            } else {
                eprintln!("Game '{}' not found!", name);
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

    Ok(())
}
