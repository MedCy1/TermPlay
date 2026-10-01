# Tetris: tetromino glow, ghost piece, hard-drop flash.
# Pieces are random, so a line clear (flash, particles, shake) is down to luck:
# re-record or tweak the moves below until one shows up.
Output docs/tetris.gif
Source scripts/vhs/_common.tape

Hide
Type "./target/release/termplay game tetris"
Enter
Sleep 1s
Show

Sleep 1s
# Drop pieces across the whole width so rows fill up evenly
Left 4
Sleep 400ms
Space
Sleep 500ms
Up
Left 2
Sleep 400ms
Space
Sleep 500ms
Right 4
Sleep 400ms
Space
Sleep 500ms
Up
Right 2
Sleep 400ms
Space
Sleep 500ms
Left 3
Sleep 400ms
Space
Sleep 500ms
Right 1
Sleep 400ms
Space
Sleep 500ms
Up
Left 4
Sleep 400ms
Space
Sleep 500ms
Right 3
Sleep 400ms
Space
Sleep 1s

Type "q"
Sleep 500ms
