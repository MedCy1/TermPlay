# Snake: smooth head-to-tail gradient, glow, food pulse, and the game-over burst.
# Food is random, so the eat-particles may or may not show; the wall crash always does.
Output docs/snake.gif
Source scripts/vhs/_common.tape

Hide
Type "./target/release/termplay game snake"
Enter
Sleep 1s
Show

# Wander a little so the gradient body gets long enough to read
Sleep 1.5s
Down
Sleep 1s
Left
Sleep 1s
Up
Sleep 1s
Right
Sleep 1s
Down
Sleep 800ms
Right

# Then run into the wall: burst, screen shake, fade to dark
Sleep 10s

Type "q"
Sleep 500ms
