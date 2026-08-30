//! Crate cleaning logic (runs `cargo clean`).

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use std::fs;

/// Domain errors produced during a `cargo clean` run.
#[derive(Debug, thiserror::Error)]
pub enum CleanError {
    /// `cargo clean` exited with a non-zero status code.
    #[error("cargo clean failed (status {status}): {stderr}")]
    CargoFailed { status: i32, stderr: String },

    /// The child process could not be spawned at all.
    #[error("failed to spawn `cargo clean`: {0}")]
    SpawnFailed(#[source] std::io::Error),
}

/// The outcome of a single `clean_crate` call.
///
/// This type cannot represent contradictory states such as "success with an
/// error" or "failure without one". Use pattern-matching to distinguish the
/// three meaningful cases.
#[derive(Debug)]
pub enum CleanOutcome {
    /// `cargo clean` completed successfully.
    Cleaned,
    /// Execution was skipped (dry-run mode); no process was launched.
    DryRun,
    /// The clean attempt failed.
    Failed(CleanError),
}

impl CleanOutcome {
    /// Returns `true` when the outcome counts as a successful clean.
    pub fn is_success(&self) -> bool {
        matches!(self, CleanOutcome::Cleaned | CleanOutcome::DryRun)
    }

    /// Returns a human-readable error string if the outcome is a failure.
    pub fn error_message(&self) -> Option<String> {
        if let CleanOutcome::Failed(e) = self {
            Some(e.to_string())
        } else {
            None
        }
    }
}

/// Result of cleaning a crate — path, timing, and outcome are always present.
pub struct CleanResult {
    pub path: PathBuf,
    pub duration: Duration,
    pub outcome: CleanOutcome,
}

impl CleanResult {
    /// Convenience: was this a successful (or dry-run) clean?
    pub fn is_success(&self) -> bool {
        self.outcome.is_success()
    }

    /// Convenience: error message string if the outcome was a failure.
    pub fn error_message(&self) -> Option<String> {
        self.outcome.error_message()
    }
}

/// Clean a Rust crate by running `cargo clean` in its directory.
///
/// In dry-run mode the function returns immediately without spawning any
/// process, preserving path and duration information.
pub async fn clean_crate(path: PathBuf, dry_run: bool) -> CleanResult {
    let start = Instant::now();
    if dry_run {
        return CleanResult {
            path,
            duration: start.elapsed(),
            outcome: CleanOutcome::DryRun,
        };
    }

    let output = Command::new("cargo")
        .arg("clean")
        .current_dir(&path)
        .output()
        .await;

    let duration = start.elapsed();

    let outcome = match output {
        Err(e) => CleanOutcome::Failed(CleanError::SpawnFailed(e)),
        Ok(out) if out.status.success() => CleanOutcome::Cleaned,
        Ok(out) => CleanOutcome::Failed(CleanError::CargoFailed {
            status: out.status.code().unwrap_or(-1),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        }),
    };

    CleanResult { path, duration, outcome }
}

/// Recursively compute the size of a directory (in bytes).
pub fn dir_size(path: &PathBuf) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    size += metadata.len();
                }
            } else if path.is_dir() {
                size += dir_size(&path);
            }
        }
    }
    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    // ── dry-run ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn dry_run_returns_dry_run_outcome() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname='a'\nversion='0.1.0'\nedition='2021'",
        )
        .unwrap();
        let result = clean_crate(dir.path().to_path_buf(), true).await;
        assert!(
            matches!(result.outcome, CleanOutcome::DryRun),
            "expected DryRun, got {:?}",
            result.outcome
        );
        assert!(result.is_success());
        assert!(result.error_message().is_none());
    }

    // ── spawn failure ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn spawn_failure_produces_failed_outcome() {
        // An empty directory has no Cargo.toml; cargo will exit non-zero.
        // To test SpawnFailed we can't easily make execve fail portably, so
        // we test the non-zero-exit path instead (CargoFailed).
        let dir = tempdir().unwrap();
        // No Cargo.toml → `cargo clean` exits non-zero.
        let result = clean_crate(dir.path().to_path_buf(), false).await;
        assert!(
            matches!(result.outcome, CleanOutcome::Failed(_)),
            "expected Failed, got {:?}",
            result.outcome
        );
        assert!(!result.is_success());
        assert!(result.error_message().is_some());
    }

    // ── contradictory states are impossible ───────────────────────────────

    #[test]
    fn outcome_cannot_be_success_with_error() {
        // DryRun is success AND has no error.
        let o = CleanOutcome::DryRun;
        assert!(o.is_success());
        assert!(o.error_message().is_none());

        // Cleaned is success AND has no error.
        let o = CleanOutcome::Cleaned;
        assert!(o.is_success());
        assert!(o.error_message().is_none());

        // Failed is NOT success AND always has an error message.
        let o = CleanOutcome::Failed(CleanError::CargoFailed {
            status: 1,
            stderr: "oops".into(),
        });
        assert!(!o.is_success());
        assert!(o.error_message().is_some());
    }
}