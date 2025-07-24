//! Directory walker for finding Rust crates.

use std::path::{Path, PathBuf};
use anyhow::Result;
use ignore::WalkBuilder;
use indicatif::ProgressBar;

/// Recursively walk the directory tree and yield paths to Rust crates.
pub async fn walk_crates(
    root: PathBuf,
    max_depth: Option<usize>,
    progress: Option<&ProgressBar>,
) -> Result<Vec<PathBuf>> {
    let mut crates = Vec::new();
    let mut builder = WalkBuilder::new(&root);
    builder.max_depth(max_depth);
    builder.git_ignore(true).hidden(false).follow_links(false);
    let walker = builder.build();
    for result in walker {
        let entry = result?;
        let path = entry.path();
        if let Some(pb) = progress {
            pb.inc(1);
        }
        if path.is_dir() && is_crate_dir(path) {
            crates.push(path.to_path_buf());
        }
    }
    Ok(crates)
}

fn is_crate_dir(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use indicatif::ProgressBar;

    #[tokio::test]
    async fn test_walk_crates_finds_crates() {
        let dir = tempdir().unwrap();
        let crate1 = dir.path().join("crate1");
        let crate2 = dir.path().join("crate2");
        fs::create_dir_all(&crate1).unwrap();
        fs::create_dir_all(&crate2).unwrap();
        fs::write(crate1.join("Cargo.toml"), "[package]\nname = 'a'\nversion = '0.1.0'").unwrap();
        fs::write(crate2.join("Cargo.toml"), "[package]\nname = 'b'\nversion = '0.1.0'").unwrap();
        let pb = ProgressBar::hidden();
        let crates = walk_crates(dir.path().to_path_buf(), None, Some(&pb)).await.unwrap();
        assert_eq!(crates.len(), 2);
        assert!(crates.contains(&crate1));
        assert!(crates.contains(&crate2));
    }
} 