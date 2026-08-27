mod cli;
use cli::Cli;
use clap::Parser;
use std::sync::Arc;
use colored::*;
use std::thread;
use std::time::Duration;

use cargo_scrub::engine::{
    clean_selected, discover_crates, detection_spinner, cleaning_progress_bar, ScrubOptions,
};
use cargo_scrub::filter::CrateFilter;
use cargo_scrub::config::{load_config, AppConfig};
use cargo_scrub::report::format_size;
use cargo_scrub::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = if let Some(ref path) = cli.config {
        load_config(path).unwrap_or_default()
    } else {
        AppConfig::default()
    };
    // TODO: Merge config and CLI (CLI takes precedence)
    init_logging(cli.log_level.clone(), cli.quiet);

    let filter = CrateFilter::from_options(cli.filter.as_deref(), None).unwrap();
    let options = ScrubOptions {
        root: cli.path.clone(),
        max_depth: cli.max_depth,
        dry_run: cli.dry_run,
        jobs: cli.jobs,
        skip_workspaces: cli.skip_workspaces,
        workspace_mode: cli.workspace_mode,
        filter,
        selected: None,
    };

    let detect_pb = detection_spinner();
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

    let crates = discover_crates(&options, Some(detect_pb_ref), None).await?;
    let msg = format!("Detection complete. Found {} projects.", crates.len());
    detect_pb.finish_with_message(msg);
    let _ = detect_tick.join();

    if crates.is_empty() {
        println!("{}", "No Rust projects found to clean.".yellow());
        return Ok(());
    }

    if cli.check {
        println!("{}", "Rust projects that would be cleaned:".cyan());
        for info in &crates {
            println!("  {}", info.path.display());
        }
        println!("{}", format!("Total: {}", crates.len()).bold());
        return Ok(());
    }

    let paths: Vec<_> = crates.iter().map(|c| c.path.clone()).collect();
    let pb = Arc::new(cleaning_progress_bar(paths.len()));
    let (report, reclaimed) = clean_selected(
        &options,
        paths,
        None,
        cli.interactive,
        Some(pb),
    )
    .await?;

    report.print_summary();
    println!("{} {}", "Total cleaned space:".bold(), format_size(reclaimed));
    Ok(())
}
