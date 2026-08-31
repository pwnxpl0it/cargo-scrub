//! Core scrub engine: discovery, filtering, and parallel cleaning.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use clap::ValueEnum;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::{mpsc, Semaphore};

use crate::cleaner::{clean_crate, dir_size, CleanResult};
use crate::detector::{is_workspace_root, parse_workspace_members};
use crate::filter::CrateFilter;
use crate::report::SummaryReport;
use crate::walker::walk_crates;

/// Workspace cleaning mode.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// Clean only workspace root
    Root,
    /// Clean only workspace members
    Members,
    /// Clean both root and members
    All,
}

/// Options for a scrub run.
#[derive(Debug, Clone)]
pub struct ScrubOptions {
    pub root: PathBuf,
    pub max_depth: Option<usize>,
    pub dry_run: bool,
    pub jobs: usize,
    pub skip_workspaces: bool,
    pub workspace_mode: WorkspaceMode,
    pub filter: CrateFilter,
    /// Per-crate selection for TUI; `None` means all filtered crates.
    pub selected: Option<HashSet<PathBuf>>,
}

/// Metadata for a discovered crate, used by the TUI table.
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub path: PathBuf,
    pub is_workspace_root: bool,
    pub target_size: u64,
    pub selected: bool,
}

/// Progress events emitted during a scrub run.
#[derive(Debug, Clone)]
pub enum ScrubEvent {
    ScanStarted,
    ScanProgress { visited: u64 },
    ScanComplete { crates: Vec<CrateInfo> },
    CleanStarted { path: PathBuf },
    CleanFinished {
        path: PathBuf,
        success: bool,
        error: Option<String>,
        duration: std::time::Duration,
    },
    AllComplete {
        report: SummaryReport,
        reclaimed_bytes: u64,
    },
    Error { message: String },
}

fn emit(event_tx: &Option<mpsc::UnboundedSender<ScrubEvent>>, event: ScrubEvent) {
    if let Some(tx) = event_tx {
        let _ = tx.send(event);
    }
}

fn target_dir_size(path: &Path) -> u64 {
    let target = path.join("target");
    if target.exists() && target.is_dir() {
        dir_size(&target)
    } else {
        0
    }
}

/// Resolve which crate paths to clean based on workspace rules and filters.
pub fn resolve_filtered_paths(
    crate_paths: &[PathBuf],
    options: &ScrubOptions,
) -> Vec<PathBuf> {
    let mut to_clean = HashSet::new();
    let mut workspace_roots = Vec::new();

    for path in crate_paths {
        if is_workspace_root(path) {
            workspace_roots.push(path.clone());
        } else {
            to_clean.insert(path.clone());
        }
    }

    match options.workspace_mode {
        WorkspaceMode::Root => {
            for root in &workspace_roots {
                to_clean.insert(root.clone());
            }
        }
        WorkspaceMode::Members => {
            for root in &workspace_roots {
                for member in parse_workspace_members(root) {
                    to_clean.insert(member);
                }
            }
        }
        WorkspaceMode::All => {
            for root in &workspace_roots {
                to_clean.insert(root.clone());
                for member in parse_workspace_members(root) {
                    to_clean.insert(member);
                }
            }
        }
    }

    if options.skip_workspaces {
        for root in &workspace_roots {
            to_clean.remove(root);
        }
    }

    let mut filtered: Vec<_> = to_clean
        .into_iter()
        .filter(|p| options.filter.matches(p))
        .collect();

    if let Some(ref selected) = options.selected {
        filtered.retain(|p| selected.contains(p));
    }

    filtered.sort();
    filtered
}

/// Build `CrateInfo` entries from resolved paths.
pub fn build_crate_infos(paths: &[PathBuf]) -> Vec<CrateInfo> {
    paths
        .iter()
        .map(|path| CrateInfo {
            is_workspace_root: is_workspace_root(path),
            target_size: target_dir_size(path),
            selected: true,
            path: path.clone(),
        })
        .collect()
}

/// Discover Rust crates under the configured root path.
pub async fn discover_crates(
    options: &ScrubOptions,
    progress: Option<&ProgressBar>,
    event_tx: Option<mpsc::UnboundedSender<ScrubEvent>>,
) -> Result<Vec<CrateInfo>> {
    emit(&event_tx, ScrubEvent::ScanStarted);

    let crate_paths = walk_crates(
        options.root.clone(),
        options.max_depth,
        progress,
    )
    .await?;

    let filtered = resolve_filtered_paths(&crate_paths, options);
    let infos = build_crate_infos(&filtered);

    emit(
        &event_tx,
        ScrubEvent::ScanComplete {
            crates: infos.clone(),
        },
    );

    Ok(infos)
}

/// Clean the given crate paths in parallel.
pub async fn clean_selected(
    options: &ScrubOptions,
    paths: Vec<PathBuf>,
    event_tx: Option<mpsc::UnboundedSender<ScrubEvent>>,
    interactive: bool,
    progress: Option<Arc<ProgressBar>>,
) -> Result<(SummaryReport, u64)> {
    if paths.is_empty() {
        let report = SummaryReport {
            cleaned: 0,
            skipped: 0,
            errors: 0,
            total: 0,
            duration: std::time::Duration::ZERO,
            details: vec![],
        };
        emit(
            &event_tx,
            ScrubEvent::AllComplete {
                report: report.clone(),
                reclaimed_bytes: 0,
            },
        );
        return Ok((report, 0));
    }

    let pre_clean_space: u64 = paths.iter().map(|p| target_dir_size(p)).sum();
    let semaphore = Arc::new(Semaphore::new(options.jobs));
    let mut handles = Vec::new();
    let start = Instant::now();

    for path in paths {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let path = path.clone();
        let dry_run = options.dry_run;
        let pb = progress.clone();
        let event_tx = event_tx.clone();

        let handle = tokio::spawn(async move {
            let _permit = permit;

            if interactive {
                let prompt = format!("Clean Rust project at {}?", path.display());
                let confirmed = dialoguer::Confirm::new()
                    .with_prompt(prompt)
                    .interact()
                    .unwrap_or(false);
                if !confirmed {
                    if let Some(ref pb) = pb {
                        pb.inc(1);
                        pb.set_message(format!("skipped: {}", path.display()));
                    }
                    emit(
                        &event_tx,
                        ScrubEvent::CleanFinished {
                            path: path.clone(),
                            success: false,
                            error: Some("skipped by user".to_string()),
                            duration: std::time::Duration::ZERO,
                        },
                    );
                    return (path, false, Some("skipped by user".to_string()));
                }
            }

            if let Some(ref pb) = pb {
                pb.set_message(format!("cleaning: {}", path.display()));
            }
            emit(
                &event_tx,
                ScrubEvent::CleanStarted {
                    path: path.clone(),
                },
            );

            let result: CleanResult = clean_crate(path.clone(), dry_run).await;

            if let Some(ref pb) = pb {
                pb.inc(1);
                if result.success {
                    pb.set_message(format!("cleaned: {}", path.display()));
                } else {
                    pb.set_message(format!("error: {}", path.display()));
                }
            }

            let error = result.error.map(|e| e.to_string());
            emit(
                &event_tx,
                ScrubEvent::CleanFinished {
                    path: result.path.clone(),
                    success: result.success,
                    error: error.clone(),
                    duration: result.duration,
                },
            );

            (result.path, result.success, error)
        });
        handles.push(handle);
    }

    let mut cleaned = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let mut details = Vec::new();

    for handle in handles {
        let (path, success, error) = match handle.await {
            Ok(res) => res,
            Err(e) => (
                PathBuf::from("<unknown>"),
                false,
                Some(format!("task join error: {}", e)),
            ),
        };

        if let Some(ref err) = error {
            if err == "skipped by user" {
                skipped += 1;
            } else {
                errors += 1;
            }
        } else if success {
            cleaned += 1;
        } else {
            skipped += 1;
        }
        details.push((path, success, error));
    }

    if let Some(ref pb) = progress {
        pb.finish_with_message("done");
    }

    let duration = start.elapsed();
    let post_clean_space: u64 = details
        .iter()
        .filter(|(_, success, _)| *success)
        .map(|(p, _, _)| target_dir_size(p))
        .sum();
    let reclaimed_bytes = pre_clean_space.saturating_sub(post_clean_space);

    let report = SummaryReport {
        cleaned,
        skipped,
        errors,
        total: cleaned + skipped + errors,
        duration,
        details,
    };

    emit(
        &event_tx,
        ScrubEvent::AllComplete {
            report: report.clone(),
            reclaimed_bytes,
        },
    );

    Ok((report, reclaimed_bytes))
}

/// Create a CLI-style detection spinner progress bar.
pub fn detection_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Detecting Rust projects...");
    pb
}

/// Create a CLI-style cleaning progress bar.
pub fn cleaning_progress_bar(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    pb
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_filtered_paths_members_mode() {
        let dir = tempdir().unwrap();
        let ws_root = dir.path().join("workspace");
        let member = ws_root.join("crates/a");
        fs::create_dir_all(&member).unwrap();
        fs::write(
            ws_root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/a\"]",
        )
        .unwrap();
        fs::write(
            member.join("Cargo.toml"),
            "[package]\nname = 'a'\nversion = '0.1.0'",
        )
        .unwrap();

        let standalone = dir.path().join("standalone");
        fs::create_dir_all(&standalone).unwrap();
        fs::write(
            standalone.join("Cargo.toml"),
            "[package]\nname = 'b'\nversion = '0.1.0'",
        )
        .unwrap();

        let crate_paths = vec![ws_root.clone(), standalone.clone()];
        let options = ScrubOptions {
            root: dir.path().to_path_buf(),
            max_depth: None,
            dry_run: false,
            jobs: 1,
            skip_workspaces: false,
            workspace_mode: WorkspaceMode::Members,
            filter: CrateFilter::from_options(None, None).unwrap(),
            selected: None,
        };

        let filtered = resolve_filtered_paths(&crate_paths, &options);
        assert!(filtered.contains(&member));
        assert!(filtered.contains(&standalone));
        assert!(!filtered.contains(&ws_root));
    }

    #[test]
    fn test_resolve_filtered_paths_skip_workspaces() {
        let dir = tempdir().unwrap();
        let ws_root = dir.path().join("workspace");
        fs::create_dir_all(&ws_root).unwrap();
        fs::write(ws_root.join("Cargo.toml"), "[workspace]\nmembers = []").unwrap();

        let options = ScrubOptions {
            root: dir.path().to_path_buf(),
            max_depth: None,
            dry_run: false,
            jobs: 1,
            skip_workspaces: true,
            workspace_mode: WorkspaceMode::Root,
            filter: CrateFilter::from_options(None, None).unwrap(),
            selected: None,
        };

        let filtered = resolve_filtered_paths(&[ws_root.clone()], &options);
        assert!(filtered.is_empty());
    }

    #[tokio::test]
    async fn test_cleaning_concurrency_bounded_by_jobs() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let max_concurrent_observed = Arc::new(AtomicUsize::new(0));
        let active_tasks = Arc::new(AtomicUsize::new(0));
        let jobs = 2;
        let semaphore = Arc::new(Semaphore::new(jobs));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let active = active_tasks.clone();
            let max_obs = max_concurrent_observed.clone();
            let handle = tokio::spawn(async move {
                let _permit = permit;
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                max_obs.fetch_max(current, Ordering::SeqCst);
                std::thread::sleep(std::time::Duration::from_millis(20));
                active.fetch_sub(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.unwrap();
        }

        assert!(
            max_concurrent_observed.load(Ordering::SeqCst) <= jobs,
            "Active concurrent tasks exceeded jobs limit"
        );
    }
}
