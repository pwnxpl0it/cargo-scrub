use clap::Parser;
use std::ffi::OsString;
use std::path::PathBuf;
use cargo_scrub::loglevel::LogLevel;
use cargo_scrub::engine::WorkspaceMode;

/// Recursively clean Rust crates in a directory tree.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to start searching for crates (default: current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Perform a dry run without cleaning
    #[arg(long)]
    pub dry_run: bool,

    /// Suppress most output
    #[arg(long, short)]
    pub quiet: bool,

    /// Maximum directory depth to search
    #[arg(long, value_name = "N")]
    pub max_depth: Option<usize>,

    /// Number of concurrent cleaning jobs
    #[arg(long, short = 'j', value_name = "N", default_value_t = 4)]
    pub jobs: usize,

    /// Prompt before cleaning each crate
    #[arg(long)]
    pub interactive: bool,

    /// Regex to filter crates by name or path
    #[arg(long, value_name = "REGEX")]
    pub filter: Option<String>,

    /// Skip cleaning workspace roots
    #[arg(long)]
    pub skip_workspaces: bool,

    /// Execute the cleaning process (if omitted, cargo-scrub inspects and lists projects without cleaning)
    #[arg(long)]
    pub clean: bool,

    /// Only list crates that would be cleaned (legacy alias, default behavior)
    #[arg(long)]
    pub check: bool,

    /// Path to config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Set log level (info, debug, error, silent)
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    /// Launch full-screen interactive dashboard
    #[arg(long, conflicts_with = "interactive")]
    pub tui: bool,

    /// Workspace cleaning mode: root, members, or all
    #[arg(long, value_enum, default_value_t = WorkspaceMode::Members)]
    pub workspace_mode: WorkspaceMode,
}

/// Strip the subcommand name cargo injects when invoked as `cargo scrub`.
///
/// Cargo runs external subcommands as `cargo-scrub scrub [ARGS...]`, so
/// without this the literal `scrub` binds to the positional `PATH` argument
/// and cargo-scrub tries to walk a directory named `scrub`. Only the first
/// argument is considered, so `cargo scrub scrub` still targets `./scrub`.
/// Direct `cargo-scrub [ARGS...]` invocations pass through unchanged.
pub fn strip_cargo_subcommand<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    if argv.get(1).map(|a| a.to_str() == Some("scrub")).unwrap_or(false) {
        argv.remove(1);
    }
    argv
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(args: &[&str]) -> Vec<String> {
        strip_cargo_subcommand(args.iter().copied())
            .into_iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn strips_the_subcommand_cargo_injects() {
        assert_eq!(strip(&["cargo-scrub", "scrub"]), ["cargo-scrub"]);
        assert_eq!(
            strip(&["cargo-scrub", "scrub", "--tui"]),
            ["cargo-scrub", "--tui"]
        );
    }

    #[test]
    fn leaves_direct_invocations_alone() {
        assert_eq!(
            strip(&["cargo-scrub", "--tui", "--dry-run"]),
            ["cargo-scrub", "--tui", "--dry-run"]
        );
        assert_eq!(
            strip(&["cargo-scrub", "/tmp/crates"]),
            ["cargo-scrub", "/tmp/crates"]
        );
    }

    /// `cargo scrub scrub` must still target the `./scrub` directory: only
    /// the argument cargo injects is removed, not every occurrence.
    #[test]
    fn keeps_a_path_that_is_itself_named_scrub() {
        assert_eq!(
            strip(&["cargo-scrub", "scrub", "scrub"]),
            ["cargo-scrub", "scrub"]
        );
    }

    #[test]
    fn handles_short_argv_without_panicking() {
        assert_eq!(strip(&["cargo-scrub"]), ["cargo-scrub"]);
        assert!(strip(&[]).is_empty());
    }

    /// The parsed CLI must agree with the stripping: `cargo scrub` targets the
    /// current directory, not a directory called `scrub`.
    #[test]
    fn parsed_path_defaults_to_cwd_under_cargo() {
        let cli = Cli::parse_from(strip_cargo_subcommand(["cargo-scrub", "scrub"]));
        assert_eq!(cli.path, PathBuf::from("."));
    }
}
