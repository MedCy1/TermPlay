# Pong: neon borders, Braille ball with motion trail, paddle sparks, and a goal burst.
# Fixed seed 17: the ball first heads for the player, the paddle is moved offline-planned
# (3 up, then 1 down), you return the ball twice (blue sparks), and the AI misses the second
# return, which triggers the goal burst and shake on the right edge.
# Paddle positions only matter at the moment of impact, so timings tolerate ~100 ms of jitter.
Output docs/pong.gif
Source scripts/vhs/_common.tape
Env TERMPLAY_SEED "17"

Type "./target/release/termplay game pong"
Enter
Sleep 1s
# Mode menu: single player is preselected
Enter
Show

Sleep 50ms
Type@50ms "www"
Sleep 4500ms
Type "s"

# Wait for the second return and the goal (tick ~412 = 10.3 s), then a beat to enjoy it.
Sleep 6600ms
Hide
Type "q"
Sleep 300ms
