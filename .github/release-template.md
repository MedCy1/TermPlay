# TermPlay v{VERSION}

## 🎮 New Features

- 

## 🐛 Bug Fixes

- 

## ⚡ Performance Improvements

- 

## 🔧 Technical Enhancements

- 

## 📱 Supported Platforms

This release includes pre-compiled binaries for:

- **Linux x86_64** (`termplay-linux-x86_64.tar.gz`)
- **Linux ARM64** (`termplay-linux-aarch64.tar.gz`) 
- **Windows x86_64** (`termplay-windows-x86_64.zip`)
- **macOS Intel** (`termplay-macos-x86_64.tar.gz`)
- **macOS Apple Silicon** (`termplay-macos-aarch64.tar.gz`)

## 📥 Installation

### Quick Install (Linux/macOS)

```bash
# Download and install automatically
curl -sSL https://github.com/MedCy1/TermPlay/releases/latest/download/install.sh | bash
```

### Manual Installation

1. Download the binary for your platform above
2. Extract the archive
3. Place the binary in your PATH

### From Source

```bash
git clone https://github.com/MedCy1/TermPlay.git
cd termplay
cargo install --path .
```

## 🚀 Usage

```bash
# Launch the main menu
termplay

# Launch a game directly
termplay game snake
termplay game tetris

# List all available games
termplay list

# Help
termplay --help
```

## 🎵 Audio Features

This version includes:
- 🎼 Background music for each game
- 🔊 Immersive sound effects
- 🎚️ In-game audio controls (M for effects, N for music)

## 🏆 Score System

- Secure local leaderboards
- Anti-cheat protection
- Progress statistics
- Automatic settings backup

## 🐛 Known Issues

- 

## 🔄 Migration from Previous Version

Settings and scores are migrated automatically. If you encounter issues:

```bash
# Reset settings
termplay --reset-config

# Diagnose audio issues
termplay --audio-test
```

## 💬 Support and Feedback

- 🐛 **Bugs**: [Open an issue](https://github.com/MedCy1/TermPlay/issues)
- 💡 **Suggestions**: [Discussions](https://github.com/MedCy1/TermPlay/discussions)
- 📧 **Contact**: support@zer0dev.me

## 🙏 Acknowledgements

Thank you to all contributors and testers who made this release possible!

---

**SHA256 Checksums**:
```
# Checksums will be added automatically by CI
```