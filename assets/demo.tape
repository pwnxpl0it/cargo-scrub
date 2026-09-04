# cargo-scrub demo — records two things:
#   1. a basic `cargo scrub` run (inspect only, no cleaning)
#   2. TUI mode: navigate the list, select one crate, clean it
#
# Requires: vhs, ffmpeg, ttyd, and a Chromium-based browser.
#   vhs assets/demo.tape        # run from the repo root
#
# The hidden setup reads two environment variables so the same tape works
# locally and on CI:
#   REPO_ROOT  → where the repo (with a built `target/release/cargo-scrub`) lives
#   DEMO_TREE  → a directory containing at least three Rust crates to discover
# Locally these default to the developer layout; CI sets them explicitly and
# stages throwaway fixture crates under `.vhs-fixtures/`.
#
# SAFETY: the TUI segment below really cleans the one selected crate. This is
# only safe because CI stages throwaway fixtures under DEMO_TREE; add
# --dry-run to the "cargo scrub --tui" line when recording against real
# crates locally (the frame will then report "could clean" instead of
# deleting anything).

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
Type "export PATH=${REPO_ROOT:-$HOME/personal/cargo-scrub}/target/release:$PATH"
Enter
Type "cd ${DEMO_TREE:-$HOME/personal}"
Enter
Type "clear"
Enter
Show

# ── 1. basic run: discover crates and report reclaimable space ──
Type "cargo scrub"
Enter
Sleep 4s

# ── 2. TUI mode: select one crate and really clean it ──
Type "cargo scrub --tui"
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
