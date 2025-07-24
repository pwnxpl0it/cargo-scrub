//! Config file support for cargo-scrub.

use std::path::PathBuf;
use std::fs;
use anyhow::Result;

/// Application configuration (from CLI or config file).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default, PartialEq)]
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
pub fn load_config(path: &PathBuf) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_config_from_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "jobs = 8\ndry_run = true").unwrap();
        let config = load_config(&file.path().to_path_buf()).unwrap();
        assert_eq!(config.jobs, Some(8));
        assert_eq!(config.dry_run, Some(true));
    }

    #[test]
    fn test_load_config_missing_file() {
        let path = PathBuf::from("/nonexistent/config.toml");
        let config = load_config(&path).unwrap();
        assert_eq!(config, AppConfig::default());
    }
} 