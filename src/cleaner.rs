//! Crate cleaning logic (runs `cargo clean`).

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;

/// Result of cleaning a crate.
pub struct CleanResult {
    pub path: PathBuf,
    pub duration: Duration,
    pub success: bool,
    pub error: Option<anyhow::Error>,
}

/// Clean a Rust crate by running `cargo clean` in its directory.
pub async fn clean_crate(
    path: PathBuf,
    dry_run: bool,
) -> CleanResult {
    let start = Instant::now();
    if dry_run {
        return CleanResult {
            path,
            duration: start.elapsed(),
            success: true,
            error: None,
        };
    }
    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(&path)
        .output()
        .await;
    let duration = start.elapsed();
    match output {
        Ok(out) if out.status.success() => CleanResult {
            path,
            duration,
            success: true,
            error: None,
        },
        Ok(out) => CleanResult {
            path,
            duration,
            success: false,
            error: Some(anyhow::anyhow!(
                "cargo clean failed: {}",
                String::from_utf8_lossy(&out.stderr)
            )),
        },
        Err(e) => CleanResult {
            path,
            duration,
            success: false,
            error: Some(anyhow::Error::from(e)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_clean_crate_dry_run() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='a'\nversion='0.1.0'").unwrap();
        let result = clean_crate(dir.path().to_path_buf(), true).await;
        assert!(result.success);
        assert!(result.error.is_none());
    }
} 