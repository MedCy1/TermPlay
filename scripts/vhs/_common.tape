# Shared settings, pulled in by each tape with `Source scripts/vhs/_common.tape`.
# Run VHS from the repository root (`make demos`).
Set Shell "bash"
Set FontSize 16
Set Width 1100
Set Height 720
Set Padding 12
Set Theme "Catppuccin Mocha"
Set Framerate 30
Set TypingSpeed 40ms

# Make sure TermPlay renders in 24-bit colour, as it does in a modern terminal.
Env COLORTERM "truecolor"
Env TERM "xterm-256color"

# Isolated profile: never read or write the real ~/.config/termplay (high scores, settings).
Env HOME "/tmp/termplay_demo"
Env XDG_CONFIG_HOME "/tmp/termplay_demo/.config"

# Terminal is 105x36 with these settings; the scripts' timings are calibrated for it.
# Each tape sets its own TERMPLAY_SEED so piece/food draws are identical on every run.
Hide
Type "rm -rf /tmp/termplay_demo && mkdir -p /tmp/termplay_demo/.config && clear"
Enter
Sleep 300ms
