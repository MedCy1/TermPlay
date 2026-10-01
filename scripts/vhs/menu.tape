# Main menu: colour wave across the banner, drifting ambient dust, gliding selector,
# pulsing chevron, and the game preview card changing as you move through the list.
Output docs/menu.gif
Source scripts/vhs/_common.tape

Type "./target/release/termplay"
Enter
Sleep 350ms
Show

# Let the wave and the dust breathe
Sleep 2500ms
Down
Sleep 500ms
Down
Sleep 500ms
Up
Sleep 500ms
Up
Sleep 700ms

# Into the games list: the preview card follows the selection
Enter
Sleep 1s
Down
Sleep 900ms
Down
Sleep 900ms
Down
Sleep 900ms
Down
Sleep 900ms
Up
Sleep 900ms
Sleep 600ms

Hide
Type "q"
Sleep 300ms
Type "q"
Sleep 300ms
