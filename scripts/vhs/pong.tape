# Pong: neon borders, Braille ball with motion trail, paddle sparks.
Output docs/pong.gif
Source scripts/vhs/_common.tape

Hide
Type "./target/release/termplay game pong"
Enter
Sleep 1s
# Mode menu: single player is preselected
Enter
Show

Sleep 1s
# Nudge the left paddle up and down while the ball bounces off the borders
Type@120ms "wwwwwww"
Sleep 800ms
Type@120ms "sssssssss"
Sleep 800ms
Type@120ms "wwwwwwww"
Sleep 800ms
Type@120ms "ssssssss"
Sleep 3s

Type "q"
Sleep 500ms
