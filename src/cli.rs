use clap::Parser;
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

    /// Only list crates that would be cleaned
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
