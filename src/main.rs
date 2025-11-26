mod app;
mod audio;
mod cli;
mod config;
mod core;
mod games;
mod highscores;
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

#[cfg(feature = "self-update")]
use axoupdater::AxoUpdater;

/// Fonction de nettoyage d'urgence du terminal
fn emergency_terminal_cleanup() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = io::stdout().flush();
}

/// Gère la mise à jour du programme via axoupdater
#[cfg(feature = "self-update")]
fn handle_update(check_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::runtime::Runtime;

    println!("Checking for updates...");

    let rt = Runtime::new()?;
    let mut updater = AxoUpdater::new_for("termplay");

    if check_only {
        println!("⚠️  Check-only mode is not supported yet.");
        println!("Run 'termplay update' to check and install updates.");
        Ok(())
    } else {
        rt.block_on(async {
            match updater.run().await {
                Ok(Some(update)) => {
                    println!("✅ Successfully updated to version {}", update.new_version);
                    println!("Please restart the application to use the new version.");
                }
                Ok(None) => {
                    println!("✅ You are already using the latest version!");
                }
                Err(e) => {
                    eprintln!("❌ Failed to update: {}", e);
                    return Err(e.into());
                }
            }
            Ok(())
        })
    }
}

#[cfg(not(feature = "self-update"))]
fn handle_update(_check_only: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Self-update feature is not available in this build.");
    println!("Please download the latest version from:");
    println!("https://github.com/MedCy1/TermPlay/releases");
    Ok(())
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
        Some(Commands::Update { check_only }) => {
            handle_update(check_only)?;
        }
        None => {
            app.run_menu()?;
        }
    }

    // Nettoyer le hook de panic à la sortie normale
    let _ = std::panic::take_hook();

    Ok(())
}
