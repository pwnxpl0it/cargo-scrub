//! Config file support for cargo-scrub.

use std::path::PathBuf;

/// Application configuration (from CLI or config file).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AppConfig {
    pub path: Option<PathBuf>,
    pub dry_run: Option<bool>,
    pub quiet: Option<bool>,
    pub max_depth: Option<usize>,
    pub jobs: Option<usize>,
    pub interactive: Option<bool>,
    pub filter: Option<String>,
    pub skip_workspaces: Option<bool>,
    pub check: Option<bool>,
    pub log_level: Option<String>,
}

/// Load configuration from a TOML file.
pub fn load_config(path: &PathBuf) -> anyhow::Result<AppConfig> {
    // TODO: Implement config loading
    unimplemented!()
} 