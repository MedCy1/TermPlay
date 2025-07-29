# 🎮 TermPlay

**TermPlay** is a collection of stylish terminal mini-games like **Snake**, **Tetris**, and more — all playable directly from your command line.

Built in **Rust** for speed and safety, using:
- **Crossterm** for cross-platform terminal handling
- **Ratatui** for modern, colorful TUI layouts
it’s cross-platform (Linux, Windows, macOS), fast, and fun.

---

## 🚀 Features

- Cross-platform CLI games
- Retro but modern UI (colors, layout, animations)
- Modular and easy to extend
- One single command to launch:
    ```bash
    termplay         # launch game menu
    termplay snake   # launch Snake directly
    ```

## 🛠 Installation

```bash
git clone https://github.com/MedCy1/TermPlay.git
cd TermPlay
cargo build --release
./target/release/termplay
```

## 🕹 Games (so far)

- ✅ Snake
- 🔜 Tetris
- 🔜 Pong

## 📜 License

MIT

Made with ❤️ by [@mederick](https://github.com/MedCy1)