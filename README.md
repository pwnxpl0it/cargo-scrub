# cargo-scrub

A polished, fast, and safe CLI tool to recursively clean Rust crates in a directory tree. Designed for maintainers, CI, and power users who want to keep their Rust projects tidy.

## Demo

![cargo-scrub discovering crates, then cleaning one from the TUI](assets/demo.gif)

A plain `cargo scrub` run — walk the tree, list every crate it finds and report
the reclaimable space — followed by TUI mode: `A` clears the default selection,
`Up/Down` navigate, `Space` selects a crate, and `c` cleans it. The TUI take
runs with `--dry-run`, which is why the summary reports `Reclaimed 0 B`.

The recording is scripted with [vhs](https://github.com/charmbracelet/vhs); the
tape lives in [`assets/demo.tape`](assets/demo.tape) and re-records with
`vhs assets/demo.tape`.

## Features

- 🚀 **Recursively walks directories to find and clean Rust crates**
- 🧹 **Runs `cargo clean` in each detected crate**
- ⚡ **Async, parallel cleaning with configurable concurrency**
- 📊 **Modern full-screen interactive TUI dashboard powered by ratatui**
- 🎛️ **Beautiful CLI with rich options (dry-run, quiet, max-depth, jobs, etc.)**
- 📝 **Supports config file (`.cargo-scrub.toml`) for persistent defaults**
- 🔍 **Filter crates by name, path, or regex**
- 🏷️ **.gitignore-aware directory walking**
- 🧩 **Detects and handles workspaces**
- 🖥️ **Interactive prompt mode**
- 📊 **Summary report with stats and timings**
- 🖌️ **Colorful, pretty output**
- 🛠️ **Robust, extensible, and well-tested**

## Installation

```
cargo install --path .
```

Or from crates.io (when published):

```
cargo install cargo-scrub
```

## Usage

```
cargo-scrub [OPTIONS] [PATH]
```

Installed as a cargo subcommand, it is equivalent to:

```
cargo scrub [OPTIONS] [PATH]
```

### Common Options

- `--clean`             : Execute the cleaning process (omitting this lists detected crates safely)
- `--tui`               : Launch full-screen interactive TUI dashboard
- `--dry-run`           : Show what would be cleaned, but don’t actually clean
- `--quiet, -q`         : Suppress most output
- `--max-depth <N>`     : Limit directory traversal depth
- `--jobs, -j <N>`      : Number of concurrent cleaning jobs (default: 4)
- `--interactive`       : Prompt before cleaning each crate
- `--filter <REGEX>`    : Only clean crates matching regex (by name or path)
- `--skip-workspaces`   : Skip workspace roots
- `--workspace-mode`    : Workspace mode (`root`, `members`, or `all`)
- `--check`             : Only list crates that would be cleaned (default behavior)
- `--config <FILE>`     : Load options from a config file
- `--log-level <LEVEL>` : Set log level (info, debug, error, silent)

### TUI Dashboard Mode

Launch `cargo-scrub` with `--tui` to enter the keyboard-driven dashboard:

```bash
cargo-scrub --tui
```

#### Keybindings

| Key | Review Screen | Running Screen | Summary Screen |
|---|---|---|---|
| `Up/Down`, `j/k` | Move selection | Scroll view | Scroll view |
| `g` / `G` | Jump to top / bottom | Jump to top / bottom | Jump to top / bottom |
| `Space` | Toggle crate selection | — | — |
| `a` / `A` | Select all / deselect all | — | — |
| `/` | Filter crates by path | — | — |
| `d` | Toggle dry-run mode | — | — |
| `c` / `Enter` | Start cleaning selected crates | — | — |
| `?` | Toggle help overlay | Toggle help overlay | Toggle help overlay |
| `q` / `Esc` | Quit | Quit (with confirmation) | Quit |

### Examples

- Scan and preview detected crates safely without cleaning:
  ```
  cargo-scrub
  ```
- Clean all detected crates in the current directory tree:
  ```
  cargo-scrub --clean
  ```
- Clean with 8 parallel jobs, max depth 3:
  ```
  cargo-scrub --clean --jobs 8 --max-depth 3
  ```
- Only clean crates matching `foo`:
  ```
  cargo-scrub --clean --filter foo
  ```
- Interactive mode:
  ```
  cargo-scrub --interactive
  ```

## Configuration

You can create a `.cargo-scrub.toml` file in your project or home directory to persist default options:

```toml
jobs = 8
dry_run = false
max_depth = 2
filter = "mycrate"
log_level = "info"
```

## Contributing

Contributions are welcome! Please open issues or pull requests. All code should be idiomatic, tested, and documented. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.
