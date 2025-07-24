# cargo-scrub

A polished, fast, and safe CLI tool to recursively clean Rust crates in a directory tree. Designed for maintainers, CI, and power users who want to keep their Rust projects tidy.

## Features

- 🚀 **Recursively walks directories to find and clean Rust crates**
- 🧹 **Runs `cargo clean` in each detected crate**
- ⚡ **Async, parallel cleaning with configurable concurrency**
- 🎛️ **Beautiful CLI with rich options (dry-run, quiet, max-depth, jobs, etc.)**
- 📝 **Supports config file (`.rustcleaner.toml`) for persistent defaults**
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

### Common Options

- `--dry-run`           : Show what would be cleaned, but don’t actually clean
- `--quiet, -q`         : Suppress most output
- `--max-depth <N>`     : Limit directory traversal depth
- `--jobs, -j <N>`      : Number of concurrent cleaning jobs (default: 4)
- `--interactive`       : Prompt before cleaning each crate
- `--filter <REGEX>`    : Only clean crates matching regex (by name or path)
- `--skip-workspaces`   : Skip workspace roots
- `--check`             : Only list crates that would be cleaned
- `--config <FILE>`     : Load options from a config file
- `--log-level <LEVEL>` : Set log level (info, debug, error, silent)

### Examples

- Clean all crates in the current directory tree:
  ```
  cargo-scrub
  ```
- Dry run, only show what would be cleaned:
  ```
  cargo-scrub --dry-run
  ```
- Clean with 8 parallel jobs, max depth 3:
  ```
  cargo-scrub --jobs 8 --max-depth 3
  ```
- Only clean crates matching `foo`:
  ```
  cargo-scrub --filter foo
  ```
- Interactive mode:
  ```
  cargo-scrub --interactive
  ```

## Configuration

You can create a `.rustcleaner.toml` file in your project or home directory to persist default options:

```toml
jobs = 8
dry_run = false
max_depth = 2
filter = "mycrate"
log_level = "info"
```

## Contributing

Contributions are welcome! Please open issues or pull requests. All code should be idiomatic, tested, and documented. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option. 