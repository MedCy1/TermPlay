# 2048: smooth tile slide, 3D-style tiles, merge pop with sparks.
Output docs/2048.gif
Source scripts/vhs/_common.tape

Hide
Type "./target/release/termplay game 2048"
Enter
Sleep 1s
Show

Sleep 800ms
# Cycle through the four directions; early merges are almost guaranteed
Left
Sleep 400ms
Down
Sleep 400ms
Right
Sleep 400ms
Up
Sleep 400ms
Left
Sleep 400ms
Down
Sleep 400ms
Right
Sleep 400ms
Up
Sleep 400ms
Left
Sleep 400ms
Down
Sleep 400ms
Left
Sleep 400ms
Up
Sleep 400ms
Right
Sleep 400ms
Down
Sleep 400ms
Left
Sleep 400ms
Up
Sleep 1500ms

Type "q"
Sleep 500ms
