# Snake: smooth head-to-tail gradient, glowing head and food, eat bursts, and a clean turn.
# Fixed seed 1971 puts the first three pieces of food where this key sequence reaches them.
# Turns are timed against the 300 ms tick (speeding up as the snake grows): if a run misses
# one, nudge the Sleeps below by ~50 ms.
Output docs/snake.gif
Source scripts/vhs/_common.tape
Env TERMPLAY_SEED "1971"

Type "./target/release/termplay game snake"
Enter
Sleep 350ms
Show

# First food is straight ahead (to the right): no input needed.
Sleep 2297ms
Up
Sleep 1118ms
Left
# Third food, then a sharp turn down.
Sleep 2385ms
Down
Sleep 1800ms

Hide
Type "q"
Sleep 300ms
