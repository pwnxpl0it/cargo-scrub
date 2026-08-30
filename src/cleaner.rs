//! Crate cleaning logic (runs `cargo clean`).

use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::process::Command;
use std::fs;

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

/// Recursively compute the size of a directory (in bytes).
///
/// Symlinks are not followed: a symlink entry contributes its own metadata
/// size (the link itself), not the size of the target. This matches the
/// `follow_links(false)` setting used by the crate walker and prevents
/// infinite loops through circular symlinks.
pub fn dir_size(path: &PathBuf) -> u64 {
    let mut size = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // Use symlink_metadata so we classify the entry itself, not its
            // target. Unreadable entries are silently skipped, consistent with
            // how the walker handles inaccessible directories.
            if let Ok(meta) = fs::symlink_metadata(&path) {
                let ft = meta.file_type();
                if ft.is_file() {
                    size += meta.len();
                } else if ft.is_dir() {
                    // Only recurse into real directories, not symlinked ones.
                    size += dir_size(&path);
                }
                // Symlinks (ft.is_symlink()) are intentionally skipped.
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

    #[tokio::test]
    async fn test_clean_crate_dry_run() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname='a'\nversion='0.1.0'\nedition='2021'",
        )
        .unwrap();
        let result = clean_crate(dir.path().to_path_buf(), true).await;
        assert!(result.success);
        assert!(result.error.is_none());
    }

    /// dir_size must not follow symlinks: a symlinked directory should not
    /// have its content counted, preventing double-counting and infinite loops.
    #[test]
    fn dir_size_does_not_follow_symlinks() {
        let dir = tempdir().unwrap();
        let real_sub = dir.path().join("real");
        fs::create_dir_all(&real_sub).unwrap();
        // Write a file in the real sub-directory.
        let content = b"hello world";
        fs::write(real_sub.join("file.txt"), content).unwrap();

        // Size without any symlink: should count the file.
        let size_real = dir_size(&dir.path().to_path_buf());
        assert_eq!(size_real, content.len() as u64);

        // Create a symlink to the same directory inside itself.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real_sub, dir.path().join("link")).unwrap();
            // With symlink present, size should NOT double-count.
            let size_with_link = dir_size(&dir.path().to_path_buf());
            assert_eq!(
                size_with_link, size_real,
                "dir_size must not follow symlinks"
            );
        }
    }
}