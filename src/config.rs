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
    pub workspace_mode: Option<String>,
}

impl AppConfig {
    /// Merge this config with CLI options, CLI taking precedence when explicitly specified.
    pub fn merge_with_cli(
        &self,
        path: PathBuf,
        dry_run: bool,
        quiet: bool,
        max_depth: Option<usize>,
        jobs: usize,
        interactive: bool,
        filter: Option<String>,
        skip_workspaces: bool,
        log_level: crate::loglevel::LogLevel,
    ) -> (
        PathBuf,
        bool,
        bool,
        Option<usize>,
        usize,
        bool,
        Option<String>,
        bool,
        crate::loglevel::LogLevel,
    ) {
        let final_path = if path == PathBuf::from(".") {
            self.path.clone().unwrap_or(path)
        } else {
            path
        };
        let final_dry_run = dry_run || self.dry_run.unwrap_or(false);
        let final_quiet = quiet || self.quiet.unwrap_or(false);
        let final_max_depth = max_depth.or(self.max_depth);
        let final_jobs = if jobs == 4 {
            self.jobs.unwrap_or(jobs)
        } else {
            jobs
        };
        let final_interactive = interactive || self.interactive.unwrap_or(false);
        let final_filter = filter.or_else(|| self.filter.clone());
        let final_skip_workspaces = skip_workspaces || self.skip_workspaces.unwrap_or(false);
        let final_log_level = if log_level == crate::loglevel::LogLevel::Info {
            if let Some(ref lvl_str) = self.log_level {
                lvl_str.parse::<crate::loglevel::LogLevel>().unwrap_or(log_level)
            } else {
                log_level
            }
        } else {
            log_level
        };

        (
            final_path,
            final_dry_run,
            final_quiet,
            final_max_depth,
            final_jobs,
            final_interactive,
            final_filter,
            final_skip_workspaces,
            final_log_level,
        )
    }
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
    use crate::loglevel::LogLevel;
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

    #[test]
    fn test_merge_with_cli() {
        let config = AppConfig {
            path: Some(PathBuf::from("/config/path")),
            dry_run: Some(true),
            quiet: Some(true),
            max_depth: Some(3),
            jobs: Some(8),
            interactive: Some(true),
            filter: Some("config_filter".to_string()),
            skip_workspaces: Some(true),
            check: None,
            log_level: Some("debug".to_string()),
            workspace_mode: None,
        };

        // CLI defaults should be overridden by config
        let (p, d, q, md, j, i, f, sw, ll) = config.merge_with_cli(
            PathBuf::from("."),
            false,
            false,
            None,
            4,
            false,
            None,
            false,
            LogLevel::Info,
        );
        assert_eq!(p, PathBuf::from("/config/path"));
        assert!(d);
        assert!(q);
        assert_eq!(md, Some(3));
        assert_eq!(j, 8);
        assert!(i);
        assert_eq!(f, Some("config_filter".to_string()));
        assert!(sw);
        assert_eq!(ll, LogLevel::Debug);

        // Explicit CLI options take precedence over config
        let (p, _d, _q, md, j, _i, f, _sw, ll) = config.merge_with_cli(
            PathBuf::from("/explicit/cli"),
            false,
            false,
            Some(5),
            16,
            false,
            Some("cli_filter".to_string()),
            false,
            LogLevel::Error,
        );
        assert_eq!(p, PathBuf::from("/explicit/cli"));
        assert_eq!(md, Some(5));
        assert_eq!(j, 16);
        assert_eq!(f, Some("cli_filter".to_string()));
        assert_eq!(ll, LogLevel::Error);
    }
} 