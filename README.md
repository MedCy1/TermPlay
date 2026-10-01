# 🎮 TermPlay

A beautiful collection of **terminal mini-games** built with Rust, featuring modern graphics and smooth gameplay right in your terminal.

[![Rust)](https://img.shields.io/badge/Rust-1.88%2B-orange.svg)](https://www.rust-lang.org/)
[![Cross Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/MedCy1/TermPlay)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Version](https://img.shields.io/github/v/release/MedCy1/TermPlay)](https://github.com/MedCy1/TermPlay/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/MedCy1/TermPlay/total)](https://github.com/MedCy1/TermPlay/releases)

## 📸 Screenshots

### 🎮 Main Menu

![Main Menu](docs/menu.png)

### 🐍 Snake Game  

![Snake Game](docs/snake.png)

### 🧩 Tetris Game

![Tetris Game](docs/tetris.png)

## ✨ Features

- 🎨 **Beautiful UI** - 24-bit RGB gradients, glow and smooth 60 fps animations
- ✨ **Juicy Game Feel** - Particles, screen shake and fades on every impact (see [Visual Effects](#-visual-effects))
- ⣿ **High-Fidelity Rendering** - Braille sub-cell canvas for smooth balls and motion trails
- 🌈 **Terminal-Friendly** - Honors `NO_COLOR`, falls back to 256 colors, and can switch effects off with `--no-fx`
- 🎮 **Classic Games** - Faithful recreations of beloved retro games
- 🚀 **High Performance** - Built in Rust for speed and reliability
- 🖥️ **Cross Platform** - Works on Windows, macOS, and Linux
- 🎯 **Responsive Design** - Automatically adapts to your terminal size
- 📱 **Intuitive Controls** - Simple keyboard controls for all games
- 🏗️ **Modular Architecture** - Easy to extend with new games
- 🎵 **Rich Audio System** - Sound effects and dynamic music for immersive gameplay
- ⚙️ **Configurable Settings** - Audio controls and game preferences
- 📦 **Easy Installation** - Professional installers for all platforms
- 🔄 **Auto-Update** - Built-in update system to stay current

## ✨ Visual Effects

TermPlay ships a small shared rendering engine (`src/engine/`) that every game builds on:

- **Particle system** - Fixed-capacity pool (no allocation while playing) with lifetime, velocity, gravity and RGB colour gradients. Debris glyphs: `*` `·` `+` `×` `▀` `▄`.
- **Braille canvas** - Each terminal cell is a 2×4 grid of dots (`U+2800`–`U+28FF`), so the Breakout and Pong balls move smoothly and leave a fading motion trail. Positions are interpolated between logic ticks, so physics stay independent of the display rate.
- **Screen shake, glow and fades** - Short exponential-decay shakes on heavy impacts, soft light around active pieces, fade-to-black on game over and between the menu and games.
- **60 fps animation loop** - Games opt in with `frame_time()` / `animate(dt)`; the logic tick rate is unchanged.

| Game | Highlights |
| --- | --- |
| Snake | Gradient body, pulsing food glow, eat and crash bursts |
| Tetris | Tetromino glow, ghost piece, line-clear flash, shake scaled to lines cleared |
| Pong | Neon borders that flash on impact, Braille trail that lengthens with speed, goal bursts |
| Breakout | Braille ball trail, glowing paddle, bricks that burst in their own colour |
| 2048 | Sliding tiles, raised rounded tiles, merge pop with sparks and floating `+N` |
| Minesweeper | Wave-shaped auto-reveal, flag sparks, explosion, victory confetti |
| Game of Life | Cell-age heatmap (newborn → stable → ancient) and fading traces |

### Terminal compatibility

| Environment | Behaviour |
| --- | --- |
| `COLORTERM=truecolor` or `24bit` (Windows is assumed truecolor) | Full effects |
| No truecolor | Colours are quantised to the xterm 256-colour palette and glows are turned off |
| `NO_COLOR` set to a non-empty value, or `--no-color` | Colours stripped, effects off |
| `--no-fx` | Particles, screen shake, glow and fades off; colours kept |

## 🕹️ Available Games

### 🐍 Snake

Classic Snake game with modern visuals and progressive difficulty

- **Square cells** with gradient effects
- **Progressive speed** - Gets faster as you grow
- **Real-time stats** - Score, length, and current speed
- **Smooth controls** with arrow keys

### 🧩 Tetris  

Complete Tetris implementation with all classic features

- **7 authentic tetrominoes** with proper colors and rotations
- **Line clearing** with classic scoring system (40/100/300/1200 points)
- **Progressive levels** - Speed increases every 10 lines
- **Next piece preview**
- **Soft drop** (↓) and **hard drop** (Space)
- **Dynamic music** - Changes tempo based on game intensity

### 🏓 Pong

Classic arcade table tennis with AI opponent

- **AI opponent** with adaptive difficulty
- **Real-time physics** - Realistic ball movement and paddle collision
- **Speed progression** - Ball gets faster as rallies continue
- **Score tracking** - First to reach target score wins

### 🧮 2048

Number puzzle game with smooth tile sliding mechanics

- **Smooth animations** - Tiles slide and merge with visual feedback
- **Score tracking** - Current score and best score persistence
- **Game over detection** - Automatic win/loss detection
- **Undo functionality** - Mistake recovery system

### 💣 Minesweeper

Classic mine detection puzzle game

- **Customizable grid** - Adjustable field size and mine density
- **Flag system** - Mark suspected mines with flags
- **Auto-reveal** - Click empty spaces to reveal connected areas
- **Timer and counter** - Track elapsed time and remaining mines

### 🧱 Breakout

Brick-breaking arcade classic

- **Physics-based gameplay** - Realistic ball and paddle physics
- **Progressive difficulty** - Multiple levels with different brick layouts
- **Power-ups** - Special abilities and enhanced gameplay mechanics
- **Combo system** - Score multipliers for consecutive hits

### 🔬 Conway's Game of Life

Cellular automaton simulation and visualization

- **Interactive controls** - Play, pause, step-through, and reset
- **Dynamic grid** - Resizable playing field that adapts to terminal
- **Pattern editing** - Click to toggle cell states and create patterns
- **Speed control** - Adjustable simulation speed

## 🚀 Installation

### 📦 Pre-built Installers (Recommended)

#### Linux / macOS
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/MedCy1/TermPlay/releases/latest/download/termplay-installer.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://github.com/MedCy1/TermPlay/releases/latest/download/termplay-installer.ps1 | iex
```

#### macOS (Homebrew)
```bash
brew install MedCy1/tap/termplay
```

#### Windows (MSI Installer)
Download the `.msi` installer from the [latest release](https://github.com/MedCy1/TermPlay/releases/latest)

### 📥 Manual Download

Download pre-built binaries for your platform from [GitHub Releases](https://github.com/MedCy1/TermPlay/releases/latest):
- `termplay-x86_64-unknown-linux-gnu.tar.xz` - Linux (Intel/AMD)
- `termplay-aarch64-unknown-linux-gnu.tar.xz` - Linux (ARM64)
- `termplay-x86_64-pc-windows-msvc.zip` - Windows
- `termplay-x86_64-apple-darwin.tar.xz` - macOS (Intel)
- `termplay-aarch64-apple-darwin.tar.xz` - macOS (Apple Silicon)

### 🔧 From Source

```bash
git clone https://github.com/MedCy1/TermPlay.git
cd TermPlay
cargo build --release
```

Requires Rust 1.88 or newer.

### 🎮 Quick Start

```bash
# Launch the main menu
termplay

# Play a specific game directly
termplay game snake
termplay game tetris
termplay game pong
termplay game 2048
termplay game minesweeper
termplay game breakout
termplay game gameoflife

# List all available games
termplay list

# Tone down the rendering for slower or colourless terminals
termplay --no-fx game tetris      # no particles, screen shake, glow or fades
termplay --no-color               # no colours at all (same as NO_COLOR=1)

# Check for updates
termplay update
```

Game names are case-insensitive (`termplay game breakout`, `termplay game "Game of Life"` and `termplay game gameoflife` all work).

### ⚙️ Command-Line Options

| Option | Description |
| --- | --- |
| `--no-fx` | Disable particles, screen shake, glow and fade transitions |
| `--no-color` | Disable all colours (also honoured through the `NO_COLOR` environment variable) |
| `-h`, `--help` | Show help |
| `-V`, `--version` | Show the version |

Both flags are global and work with every subcommand.


## 🎮 How to Play

### Main Menu Navigation

- **↑/↓** - Navigate menu options
- **Enter** - Select option
- **Q** - Quit
- **Esc** - Go back (in submenus)

### Snake Controls

- **Arrow Keys** - Move snake
- **Q** - Quit to menu
- **R** - Restart (when game over)

### Tetris Controls

- **←/→** - Move piece left/right
- **↓** - Soft drop (faster descent + 1 point per line)
- **↑** - Rotate piece
- **Space** - Hard drop (instant drop + 2 points per line)
- **Q** - Quit to menu
- **R** - Restart (when game over)

### Pong Controls

- **↑/↓** - Move paddle up/down
- **Q** - Quit to menu
- **R** - Restart (when game over)

### 2048 Controls

- **Arrow Keys** - Slide tiles in direction
- **Q** - Quit to menu
- **R** - Restart game
- **U** - Undo last move

### Minesweeper Controls

- **Arrow Keys** - Move cursor
- **Space** - Reveal cell
- **F** - Flag/unflag cell
- **Q** - Quit to menu
- **R** - Restart game

### Breakout Controls

- **←/→** - Move paddle left/right
- **Space** - Launch ball (when paused)
- **Q** - Quit to menu
- **R** - Restart (when game over)

### Conway's Game of Life Controls

- **Space** - Play/pause simulation
- **S** - Step one generation
- **R** - Reset/clear grid
- **Arrow Keys** - Move cursor
- **Enter** - Toggle cell state
- **Q** - Quit to menu

## 🛠️ Technical Details

### Built With

- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Ratatui](https://github.com/ratatui-org/ratatui)** - Modern terminal UI library
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **[Clap](https://github.com/clap-rs/clap)** - Command line argument parsing
- **[Rand](https://github.com/rust-random/rand)** - Random number generation
- **[Rodio](https://github.com/RustAudio/rodio)** - Audio playback library for sound effects and music
- **[Serde](https://github.com/serde-rs/serde)** - Serialization framework for configuration management
- **[Dirs](https://github.com/dirs-dev/dirs-rs)** - System directory discovery

### Architecture

- **Modular Game System** - Each game implements a common `Game` trait
- **Dynamic Registration** - Games are automatically registered and discoverable
- **Responsive Rendering** - Games adapt to terminal dimensions
- **Event-Driven** - Efficient input handling with configurable tick rates
- **Shared Rendering Engine** - `src/engine/` provides particles, Braille canvas, glow, shake and fades to all games
- **Audio System** - Centralized audio management with per-game music and sound effects
- **Configuration Management** - Persistent settings with JSON-based configuration files

### Performance

- **Optimized Rendering** - Only redraws changed areas
- **Memory Efficient** - Zero-allocation hot paths where possible
- **Low Latency** - Sub-50ms input response times
- **Adaptive Refresh** - Games can control their own update frequency, or opt in to a 60 fps render loop
- **Draw Rate Floor** - At least 8 ms between frames so a held key cannot spin the CPU

## 🎯 Scoring Systems

### Snake

- **+10 points** per food eaten
- **Speed bonus** - Faster gameplay as snake grows
- Final score based on snake length and survival time

### Tetris

- **Single line:** 40 × level
- **Double lines:** 100 × level  
- **Triple lines:** 300 × level
- **Tetris (4 lines):** 1200 × level
- **Soft drop:** +1 point per line
- **Hard drop:** +2 points per line
- **Level progression:** Every 10 lines cleared

### Pong

- **+1 point** per successful return
- **First to reach target score** wins the match

### 2048

- **Tile merge value** - Points equal the value of the merged tile
- **Best score tracking** - Persistent high score storage

### Minesweeper

- **Time-based scoring** - Faster completion = higher score
- **Accuracy bonus** - Fewer mistakes = bonus points

### Breakout

- **Brick value scoring** - Different brick types have different point values
- **Combo multipliers** - Consecutive hits increase score multiplier

## 🔧 Development

### Adding New Games

1. Create a new file in `src/games/your_game.rs`
2. Implement the `Game` trait:

   ```rust
   impl Game for YourGame {
       fn handle_key(&mut self, key: KeyEvent) -> GameAction { /* ... */ }
       fn update(&mut self) -> GameAction { /* ... */ }
       fn draw(&mut self, frame: &mut Frame) { /* ... */ }
       fn tick_rate(&self) -> Duration { /* optional: logic speed */ }
       fn frame_time(&self) -> Option<Duration> { Some(Duration::from_millis(16)) } // optional: opt in to 60 fps
       fn animate(&mut self, dt: Duration) { /* optional: advance particles, shake, ... */ }
   }
   ```

3. Register in `src/games/mod.rs` (`register("Your Game", "description", ...)`)
4. Your game automatically appears in the menu!

Use `crate::engine::{particles::Particles, braille::Braille, fx}` for particles, sub-cell rendering, glow, shake and fades.

### Recording Demo GIFs

The demo scripts live in `scripts/vhs/` and are recorded with [VHS](https://github.com/charmbracelet/vhs):

```bash
make demos        # builds in release mode, then writes docs/{snake,pong,tetris,breakout,2048}.gif
```

### Quality Checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

The CI also builds against the minimum supported Rust version (`rust-version` in `Cargo.toml`).

### Building for Different Platforms

```bash
# Windows
cargo build --release --target x86_64-pc-windows-gnu

# macOS  
cargo build --release --target x86_64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu

# ARM64 (Apple Silicon, ARM Linux)
cargo build --release --target aarch64-apple-darwin
cargo build --release --target aarch64-unknown-linux-gnu
```

### Release Management

```bash
# Use the release script for automated versioning and building
./scripts/release.sh

# This script handles:
# - Version bumping
# - Cross-platform compilation
# - Asset packaging
# - Release notes generation
```

## 📋 TODO / Roadmap

### ✅ Completed

- [x] **Snake** - Classic snake game with progressive difficulty and modern graphics
- [x] **Tetris** - Complete implementation with line clearing, levels, and authentic gameplay
- [x] **Pong** - Classic paddle game with AI opponent and physics-based gameplay
- [x] **2048** - Number sliding puzzle game with smooth animations
- [x] **Minesweeper** - Classic mine detection game with customizable difficulty
- [x] **Breakout** - Brick breaking arcade game with physics and power-ups
- [x] **Conway's Game of Life** - Interactive cellular automaton visualization
- [x] **Audio System** - Complete sound effects and dynamic music system
- [x] **Menu System** - Beautiful navigation with Games, Settings, and About sections
- [x] **Configuration System** - Persistent audio and game settings
- [x] **Cross-platform Support** - Works seamlessly on Windows, macOS, and Linux (including ARM64)
- [x] **CI/CD Pipeline** - Automated building, testing, and release management
- [x] **Installation Scripts** - Easy setup and deployment tools

### 🚧 In Progress / Planned

- [x] **High Scores** - Persistent leaderboards for each game ✅
- [x] **Auto-Update System** - Built-in update mechanism ✅
- [x] **Professional Distribution** - Multi-platform installers (MSI, shell, PowerShell) ✅
- [ ] **Themes** - Customizable color schemes and visual styles
- [ ] **Game Replays** - Record and playback game sessions
- [ ] **Advanced Settings** - Per-game configuration options
- [ ] **Tournament Mode** - Compete across multiple games
- [ ] **Online Features** - Leaderboards and multiplayer capabilities

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 🐛 Bug Reports

If you encounter any bugs or have feature requests, please [open an issue](https://github.com/MedCy1/TermPlay/issues) with:

- Your operating system and terminal
- Steps to reproduce the issue
- Expected vs actual behavior
- Screenshots if applicable

## 📜 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by classic arcade games and modern terminal applications
- Built with the amazing Rust ecosystem
- Special thanks to the Ratatui and Crossterm communities

---

**Made with ❤️ by [MedCy1](https://github.com/MedCy1)**

***Enjoy playing classic games in your terminal! 🎮***
