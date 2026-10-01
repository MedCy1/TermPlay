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
