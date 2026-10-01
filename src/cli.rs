use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "termplay")]
#[command(about = "A collection of stylish terminal mini-games")]
#[command(version)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "Disable particles, screen shake, glow and fades"
    )]
    pub no_fx: bool,
    #[arg(
        long,
        global = true,
        help = "Disable all colors (also honoured via the NO_COLOR variable)"
    )]
    pub no_color: bool,
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
    #[command(about = "Check for updates and install the latest version")]
    Update {
        #[arg(long, help = "Only check for updates without installing")]
        check_only: bool,
    },
}
