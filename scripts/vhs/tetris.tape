# Tetris: tetromino glow, ghost piece, hard-drop flash, and a triple line clear
# (white flash, particles in each block's colour, screen shake).
# Fixed seed + fixed keys = the same pieces and placements every time; the placements
# were found offline for this seed with a small greedy planner.
Output docs/tetris.gif
Source scripts/vhs/_common.tape
Env TERMPLAY_SEED "14"

Type "./target/release/termplay game tetris"
Enter
Sleep 1s
Show

Sleep 700ms
Left@80ms 4
Sleep 90ms
Space
Sleep 450ms
Up
Sleep 90ms
Left@80ms 5
Sleep 90ms
Space
Sleep 450ms
Left
Sleep 90ms
Space
Sleep 450ms
Up
Sleep 90ms
Left@80ms 3
Sleep 90ms
Space
Sleep 450ms
Up@80ms 2
Sleep 90ms
Left@80ms 4
Sleep 90ms
Space
Sleep 450ms
Right
Sleep 90ms
Space
Sleep 450ms
Left
Sleep 90ms
Space
Sleep 450ms
Left
Sleep 90ms
Space
Sleep 450ms
Up
Sleep 90ms
Right@80ms 2
Sleep 90ms
Space
Sleep 450ms
Up@80ms 3
Sleep 90ms
Right@80ms 4
Sleep 90ms
Space
Sleep 450ms

# The last drop clears three lines: let the flash, sparks and shake play out.
Sleep 1500ms
Hide
Type "q"
Sleep 300ms
