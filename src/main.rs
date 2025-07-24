mod cli;
use cli::Cli;
use cargo_scrub::loglevel::LogLevel;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use dialoguer::Confirm;
use colored::*;

use cargo_scrub::walker::walk_crates;
use cargo_scrub::detector::{is_workspace_root};
use cargo_scrub::cleaner::{clean_crate, CleanResult};
use cargo_scrub::filter::CrateFilter;
use cargo_scrub::config::{load_config, AppConfig};
use cargo_scrub::report::SummaryReport;
use cargo_scrub::logging::init_logging;

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

    let crate_paths = walk_crates(root, max_depth).await?;
    let mut filtered = Vec::new();
    for path in crate_paths {
        if filter.matches(&path) {
            if skip_workspaces && is_workspace_root(&path) {
                continue;
            }
            filtered.push(path);
        }
    }
    if filtered.is_empty() {
        println!("{}", "No crates found to clean.".yellow());
        return Ok(());
    }
    if check {
        println!("{}", "Crates that would be cleaned:".cyan());
        for path in &filtered {
            println!("  {}", path.display());
        }
        println!("{}", format!("Total: {}", filtered.len()).bold());
        return Ok(());
    }
    let semaphore = Arc::new(Semaphore::new(jobs));
    let mut handles = Vec::new();
    let mut details = Vec::new();
    let start = std::time::Instant::now();
    for path in filtered {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let path = path.clone();
        let dry_run = dry_run;
        let interactive = interactive;
        let handle = tokio::spawn(async move {
            let _permit = permit;
            if interactive {
                let prompt = format!("Clean crate at {}?", path.display());
                if !Confirm::new().with_prompt(prompt).interact().unwrap_or(false) {
                    return (path, false, Some("skipped by user".to_string()));
                }
            }
            let result: CleanResult = clean_crate(path.clone(), dry_run).await;
            if result.success {
                (result.path, true, None)
            } else {
                (result.path, false, result.error.map(|e| e.to_string()))
            }
        });
        handles.push(handle);
    }
    let mut cleaned = 0;
    let mut skipped = 0;
    let mut errors = 0;
    for handle in handles {
        let (path, success, error) = handle.await?;
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
    let duration = start.elapsed();
    let report = SummaryReport {
        cleaned,
        skipped,
        errors,
        total: cleaned + skipped + errors,
        duration,
        details,
    };
    report.print_summary();
    Ok(())
}
