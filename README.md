# 🎮 TermPlay

A beautiful collection of **terminal mini-games** built with Rust, featuring modern graphics and smooth gameplay right in your terminal.

[![Rust)](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Cross Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)](https://github.com/MedCy1/TermPlay)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## 📸 Screenshots

### 🎮 Main Menu
![Main Menu](docs/menu.png)

### 🐍 Snake Game  
![Snake Game](docs/snake.png.png)

### 🧩 Tetris Game
![Tetris Game](docs/tetris.png)

## ✨ Features

- 🎨 **Beautiful UI** - Modern terminal graphics with RGB colors and smooth animations
- 🎮 **Classic Games** - Faithful recreations of beloved retro games
- 🚀 **High Performance** - Built in Rust for speed and reliability
- 🖥️ **Cross Platform** - Works on Windows, macOS, and Linux
- 🎯 **Responsive Design** - Automatically adapts to your terminal size
- 📱 **Intuitive Controls** - Simple keyboard controls for all games
- 🏗️ **Modular Architecture** - Easy to extend with new games

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
- **Ghost piece** and **wall kicks** (coming soon)

## 🚀 Installation

### From Source
```bash
git clone https://github.com/MedCy1/TermPlay.git
cd TermPlay
cargo build --release
```

### Quick Start
```bash
# Launch the main menu
./target/release/termplay

# Or play a specific game directly
./target/release/termplay snake
./target/release/termplay tetris

# List all available games
./target/release/termplay list
```

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

## 🛠️ Technical Details

### Built With
- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Ratatui](https://github.com/ratatui-org/ratatui)** - Modern terminal UI library
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Cross-platform terminal manipulation
- **[Clap](https://github.com/clap-rs/clap)** - Command line argument parsing
- **[Rand](https://github.com/rust-random/rand)** - Random number generation

### Architecture
- **Modular Game System** - Each game implements a common `Game` trait
- **Dynamic Registration** - Games are automatically registered and discoverable
- **Responsive Rendering** - Games adapt to terminal dimensions
- **Event-Driven** - Efficient input handling with configurable tick rates

### Performance
- **Optimized Rendering** - Only redraws changed areas
- **Memory Efficient** - Zero-allocation hot paths where possible
- **Low Latency** - Sub-50ms input response times
- **Adaptive Refresh** - Games can control their own update frequency

## 🎯 Scoring Systems

### Snake
- **+10 points** per food eaten
- **+1 point** per soft drop move
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

## 🔧 Development

### Adding New Games
1. Create a new file in `src/games/your_game.rs`
2. Implement the `Game` trait:
   ```rust
   impl Game for YourGame {
       fn name(&self) -> &str { "your_game" }
       fn description(&self) -> &str { "Your game description" }
       fn handle_key(&mut self, key: KeyEvent) -> GameAction { /* ... */ }
       fn update(&mut self) -> GameAction { /* ... */ }
       fn draw(&mut self, frame: &mut Frame) { /* ... */ }
       fn tick_rate(&self) -> Duration { /* optional */ }
   }
   ```
3. Register in `src/games/mod.rs`
4. Your game automatically appears in the menu!

### Building for Different Platforms
```bash
# Windows
cargo build --release --target x86_64-pc-windows-gnu

# macOS  
cargo build --release --target x86_64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu
```

## 📋 TODO / Roadmap

### ✅ Completed
- [x] **Snake** - Classic snake game with progressive difficulty and modern graphics
- [x] **Tetris** - Complete implementation with line clearing, levels, and authentic gameplay
- [x] **Menu System** - Beautiful navigation with Games, Settings, and About sections
- [x] **Cross-platform Support** - Works seamlessly on Windows, macOS, and Linux
- [x] **Pong** - Classic paddle game with AI opponent
- [x] **2048** - Number sliding puzzle game
- [x] **Minesweeper** - Classic mine detection game
- [x] **Breakout** - Brick breaking arcade game
- [x] **Conway's Game of Life** - Cellular automaton visualization

### 🚧 In Progress / Planned
- [ ] **Sound Effects** - Audio feedback for game events
  - [x] Snake
  - [x] Tetris
  - [x] Menu
  - [x] Pong
  - [x] 2048
  - [x] Minesweeper
  - [x] Breakout
  - [x] Conway's Game of Life
- [ ] **High Scores** - Persistent leaderboards
- [ ] **Themes** - Customizable color schemes
- [ ] **Replay System** - Record and playback games
- [ ] **Online Multiplayer** - Network play capabilities

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

*Enjoy playing classic games in your terminal! 🎮*