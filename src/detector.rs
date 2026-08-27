//! Crate and workspace detection utilities.

use std::path::{Path, PathBuf};
use std::fs;
use toml::Value;

/// Returns true if the given directory contains a Cargo.toml (i.e., is a Rust crate).
pub fn is_crate_dir(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
}

/// Returns true if the given directory is a workspace root.
pub fn is_workspace_root(path: &Path) -> bool {
    let cargo_toml = path.join("Cargo.toml");
    if !cargo_toml.is_file() {
        return false;
    }
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let value: Value = match content.parse::<Value>() {
        Ok(v) => v,
        Err(_) => return false,
    };
    value.get("workspace").is_some()
}

/// Parse workspace member paths from a workspace root's Cargo.toml.
pub fn parse_workspace_members(root: &Path) -> Vec<PathBuf> {
    let cargo_toml = root.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let value: Value = match content.parse::<Value>() {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let members = value.get("workspace").and_then(|ws| ws.get("members"));
    if let Some(members) = members {
        if let Some(arr) = members.as_array() {
            return arr
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| root.join(s))
                .collect();
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_is_crate_dir() {
        let dir = tempdir().unwrap();
        assert!(!is_crate_dir(dir.path()));
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='a'\nversion='0.1.0'").unwrap();
        assert!(is_crate_dir(dir.path()));
    }

    #[test]
    fn test_is_workspace_root() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[workspace]\nmembers=['a']").unwrap();
        assert!(is_workspace_root(dir.path()));
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='a'\nversion='0.1.0'").unwrap();
        assert!(!is_workspace_root(dir.path()));
    }

    #[test]
    fn test_parse_workspace_members() {
        let dir = tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]",
        )
        .unwrap();
        let members = parse_workspace_members(dir.path());
        assert_eq!(members.len(), 2);
        assert!(members.contains(&dir.path().join("crates/a")));
        assert!(members.contains(&dir.path().join("crates/b")));
    }
} 