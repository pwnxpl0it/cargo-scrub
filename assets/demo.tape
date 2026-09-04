# cargo-scrub demo — records two things:
#   1. a basic `cargo scrub` run (inspect only, no cleaning)
#   2. TUI mode: navigate the list, select one crate, clean it
#
# Requires: vhs, ffmpeg, ttyd, and a Chromium-based browser.
#   vhs assets/demo.tape        # run from the repo root
#
# SAFETY: the TUI segment below passes --dry-run, so `c` reports what it
# would clean without deleting anything. Drop --dry-run from the
# "cargo scrub --tui" line to record a take that really cleans the one
# selected crate (and only that one).

Output assets/demo.gif

Set Shell "bash"
Set FontFamily "JetBrains Mono, Noto Emoji, Noto Color Emoji"
Set FontSize 16
Set Padding 20
Set Width 1400
Set Height 470
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms
Set Framerate 20

# ── hidden setup: plain prompt, cargo-scrub on PATH, start in the crate tree ──
Hide
Type "unset PROMPT_COMMAND; PS1='> '"
Enter
Type "export PATH=$HOME/personal/cargo-scrub/target/release:$PATH"
Enter
Type "cd ~/personal"
Enter
Type "clear"
Enter
Show

# ── 1. basic run: discover crates and report reclaimable space ──
Type "cargo scrub"
Enter
Sleep 4s

# ── 2. TUI mode ──
Type "cargo scrub --tui --dry-run"
Enter
Sleep 3s

# every crate starts selected; clear the selection to pick deliberately
Type "A"
Sleep 1500ms

# navigate down to the third crate
Down
Sleep 700ms
Down
Sleep 900ms

# select it
Space
Sleep 2s

# clean the selection
Type "c"
Sleep 3s

# quit back to the shell
Type "q"
Sleep 2s
