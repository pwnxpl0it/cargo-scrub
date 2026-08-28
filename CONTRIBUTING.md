# Contributing to cargo-scrub

Thank you for your interest in contributing to **cargo-scrub**! We welcome all forms of contributions, including bug reports, feature suggestions, documentation enhancements, and pull requests.

---

## Code of Conduct

We are committed to providing a friendly, welcoming, and harassment-free experience for everyone. Please be respectful, constructive, and considerate in all interactions.

---

## How to Contribute

### 1. Reporting Bugs & Suggesting Features

- Search the existing [GitHub Issues](https://github.com/pwnxpl0it/cargo-scrub/issues) to ensure your issue or idea hasn't already been reported.
- If not, open a new issue with:
  - A clear and descriptive title.
  - Steps to reproduce the bug (including CLI flags used, OS, Rust version).
  - Expected vs. actual behavior.
  - Logs or error output where relevant.

### 2. Setting Up the Local Development Environment

Make sure you have a modern Rust toolchain installed (edition 2021, Rust 1.74+ recommended):

```bash
# Clone the repository
git clone https://github.com/pwnxpl0it/cargo-scrub.git
cd cargo-scrub

# Build the project in debug mode
cargo build

# Run unit and integration tests
cargo test

# Run with TUI mode locally
cargo run -- --tui
```

---

## Development Workflow & Guidelines

### Branching & Commits
- Create a new branch off `master` (e.g. `feature/my-feature` or `fix/my-bugfix`).
- Keep commits small, focused, and atomic.
- Write meaningful commit messages following the [Conventional Commits](https://www.conventionalcommits.org/) convention (e.g., `feat: ...`, `fix: ...`, `docs: ...`, `refactor: ...`, `test: ...`).

### Code Style & Quality Standards
- Format your code using `rustfmt`:
  ```bash
  cargo fmt --all
  ```
- Ensure zero Clippy warnings:
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- Ensure all tests pass:
  ```bash
  cargo test --all-targets
  ```

### Architecture Overview
- **`src/engine.rs`**: Core scrubbing engine, discovery, workspace resolution, and event streaming (`ScrubEvent`).
- **`src/tui/`**: Full-screen terminal dashboard powered by `ratatui` and `crossterm`.
- **`src/cleaner.rs`**: Safe `cargo clean` execution and size inspection.
- **`src/detector.rs`**: Fast crate and workspace member detection.
- **`src/config.rs`**: Configuration file parsing (`.rustcleaner.toml`) and CLI argument merging.
- **`src/walker.rs`**: Directory traversal with `.gitignore` awareness via `ignore`.

---

## Submitting Pull Requests

1. Push your branch to your fork or repository.
2. Open a Pull Request against the `master` branch.
3. Fill out the PR description template clearly explaining:
   - What changed and why.
   - Any relevant issues closed by this PR (e.g., `Fixes #12`).
   - How the changes were tested.
4. Ensure CI passes all checks.

Thank you for helping make `cargo-scrub` better!
