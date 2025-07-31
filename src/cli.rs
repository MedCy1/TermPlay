use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "termplay")]
#[command(about = "A collection of stylish terminal mini-games")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Launch a specific game directly")]
    Game {
        #[arg(help = "Name of the game to launch")]
        name: String,
    },
    #[command(about = "List all available games")]
    List,
}
