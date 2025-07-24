mod cli;
use cli::{Cli, WorkspaceMode};
use clap::Parser;
use std::sync::Arc;
use tokio::sync::Semaphore;
use dialoguer::Confirm;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;
use std::collections::HashSet;
use std::fs;

use cargo_scrub::walker::walk_crates;
use cargo_scrub::detector::{is_workspace_root};
use cargo_scrub::cleaner::{clean_crate, CleanResult, dir_size};
use cargo_scrub::filter::CrateFilter;
use cargo_scrub::config::{load_config, AppConfig};
use cargo_scrub::report::SummaryReport;
use cargo_scrub::logging::init_logging;

fn parse_workspace_members(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let cargo_toml = root.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let value: toml::Value = match content.parse::<toml::Value>() {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let members = value.get("workspace").and_then(|ws| ws.get("members"));
    if let Some(members) = members {
        if let Some(arr) = members.as_array() {
            return arr.iter().filter_map(|v| v.as_str()).map(|s| root.join(s)).collect();
        }
    }
    vec![]
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    // Load config file if specified
    let config = if let Some(ref path) = cli.config {
        load_config(path).unwrap_or_default()
    } else {
        AppConfig::default()
    };
    // TODO: Merge config and CLI (CLI takes precedence)
    let log_level = cli.log_level.clone();
    init_logging(log_level, cli.quiet);

    let root = cli.path.clone();
    let max_depth = cli.max_depth;
    let dry_run = cli.dry_run;
    let check = cli.check;
    let jobs = cli.jobs;
    let skip_workspaces = cli.skip_workspaces;
    let interactive = cli.interactive;
    let filter = CrateFilter::from_options(cli.filter.as_deref(), None).unwrap();
    let workspace_mode = cli.workspace_mode;

    // Show spinner while detecting
    let detect_pb = ProgressBar::new_spinner();
    detect_pb.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
    detect_pb.set_message("Detecting Rust projects...");
    let detect_pb_ref = &detect_pb;
    let detect_tick = thread::spawn({
        let pb = detect_pb.clone();
        move || {
            while !pb.is_finished() {
                pb.tick();
                thread::sleep(Duration::from_millis(80));
            }
        }
    });
    let crate_paths = walk_crates(root, max_depth, Some(detect_pb_ref)).await?;
    let msg = format!("Detection complete. Found {} projects.", crate_paths.len());
    detect_pb.finish_with_message(msg);
    let _ = detect_tick.join();

    let mut to_clean = HashSet::new();
    let mut workspace_roots = Vec::new();
    for path in &crate_paths {
        if is_workspace_root(path) {
            workspace_roots.push(path.clone());
        } else {
            to_clean.insert(path.clone());
        }
    }
    match workspace_mode {
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
    // Remove workspace roots if skip_workspaces is set
    if skip_workspaces {
        for root in &workspace_roots {
            to_clean.remove(root);
        }
    }
    // Filter by user filter
    let mut filtered: Vec<_> = to_clean.into_iter().filter(|p| filter.matches(p)).collect();
    filtered.sort();
    if filtered.is_empty() {
        println!("{}", "No Rust projects found to clean.".yellow());
        return Ok(());
    }
    if check {
        println!("{}", "Rust projects that would be cleaned:".cyan());
        for path in &filtered {
            println!("  {}", path.display());
        }
        println!("{}", format!("Total: {}", filtered.len()).bold());
        return Ok(());
    }
    // Calculate total size of all target/ dirs before cleaning
    let pre_clean_space: u64 = filtered.iter()
        .map(|p| {
            let target = p.join("target");
            if target.exists() && target.is_dir() {
                dir_size(&target)
            } else {
                0
            }
        })
        .sum();
    let pb = Arc::new(ProgressBar::new(filtered.len() as u64));
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
    ).unwrap()
        .progress_chars("#>-")
    );
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut handles = Vec::new();
    let mut details = Vec::new();
    let start = std::time::Instant::now();
    for path in filtered {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let path = path.clone();
        let dry_run = dry_run;
        let interactive = interactive;
        let pb = pb.clone();
        let handle = tokio::spawn(async move {
            let _permit = permit;
            if interactive {
                let prompt = format!("Clean Rust project at {}?", path.display());
                if !Confirm::new().with_prompt(prompt).interact().unwrap_or(false) {
                    pb.inc(1);
                    pb.set_message(format!("skipped: {}", path.display()));
                    return (path, false, Some("skipped by user".to_string()));
                }
            }
            pb.set_message(format!("cleaning: {}", path.display()));
            let result: CleanResult = match clean_crate(path.clone(), dry_run).await {
                r => r,
            };
            pb.inc(1);
            if result.success {
                pb.set_message(format!("cleaned: {}", path.display()));
                (result.path, true, None)
            } else {
                pb.set_message(format!("error: {}", path.display()));
                // Do not print error here, just record it for the summary
                (result.path, false, result.error.map(|e| e.to_string()))
            }
        });
        handles.push(handle);
    }
    let mut cleaned = 0;
    let mut skipped = 0;
    let mut errors = 0;
    for handle in handles {
        let (path, success, error) = match handle.await {
            Ok(res) => res,
            Err(e) => {
                // If the task itself failed (e.g., panicked), treat as error and continue
                (std::path::PathBuf::from("<unknown>"), false, Some(format!("task join error: {}", e)))
            }
        };
        if let Some(err) = &error {
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
    pb.finish_with_message("done");
    let duration = start.elapsed();
    // Calculate total size of all target/ dirs after cleaning
    let post_clean_space: u64 = details.iter()
        .filter(|(_, success, _)| *success)
        .map(|(p, _, _)| {
            let target = p.join("target");
            if target.exists() && target.is_dir() {
                dir_size(&target)
            } else {
                0
            }
        })
        .sum();
    let cleaned_space = pre_clean_space.saturating_sub(post_clean_space);
    let report = SummaryReport {
        cleaned,
        skipped,
        errors,
        total: cleaned + skipped + errors,
        duration,
        details,
    };
    report.print_summary();
    println!("{} {}", "Total cleaned space:".bold(), format_size(cleaned_space));
    Ok(())
}

// Format bytes as human-readable string
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    match bytes {
        b if b >= GB => format!("{:.2} GiB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.2} MiB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.2} KiB", b as f64 / KB as f64),
        _ => format!("{} B", bytes),
    }
}
