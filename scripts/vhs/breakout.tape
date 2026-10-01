# Breakout: Braille ball trail, glowing paddle, exploding bricks, and three clean paddle bounces.
# Breakout has no randomness, so the paddle moves below were planned offline by replaying the
# game tick by tick (50 ms): launch from the right, then steer the paddle left and right after
# each bounce. Key timings are relative to the launch and tolerate ~100 ms of jitter.
Output docs/breakout.gif
Source scripts/vhs/_common.tape
Env TERMPLAY_SEED "1"

Type "./target/release/termplay game breakout"
Enter
Sleep 350ms
Show

Sleep 250ms
# Park the paddle on the right while the ball is still stuck to it, then launch
Right@60ms 10
Sleep 300ms
Space

# First bounce is at ~0.9 s. Then glide the paddle to the left for the next one (~5.0 s).
Sleep 1150ms
Left@50ms 17
# ...and back to the right for the third bounce (~9.6 s).
Sleep 3300ms
Right@50ms 17

Sleep 5600ms
Hide
Type "q"
Sleep 300ms
