# 2048: sliding tiles, merge pops with sparks, and the floating +128 when a 128 tile appears.
# Fixed seed + fixed keys = the same game every time. The keys are a greedy
# left/down strategy found offline for this seed (a, s, d = left, down, right).
Output docs/2048.gif
Source scripts/vhs/_common.tape
Env TERMPLAY_SEED "291"

Type "./target/release/termplay game 2048"
Enter
Sleep 1s
Show

Sleep 600ms
Type@190ms "aasaaaaaaaaaaasdaaaaaaaaaaaaaaasaaaasaaadasaasaaaasaaaaaasaaaasdaasasa"

# Let the +128 float up and the sparks settle, then stop recording before the shell shows.
Sleep 1500ms
Hide
Type "q"
Sleep 300ms
