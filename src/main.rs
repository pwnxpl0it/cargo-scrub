mod cli;
mod tui;
use cli::Cli;
use clap::Parser;
use std::sync::Arc;
use colored::*;
use std::thread;
use std::time::Duration;

use cargo_scrub::engine::{
    build_clean_plan, clean_selected, discover_crates, detection_spinner, cleaning_progress_bar,
    CleanPlan, ScrubOptions,
};
use cargo_scrub::filter::CrateFilter;
use cargo_scrub::config::load_config;
use cargo_scrub::report::format_size;
use cargo_scrub::logging::init_logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = if let Some(ref path) = cli.config {
        load_config(path).unwrap_or_default()
    } else {
        let default_path = std::path::PathBuf::from(".rustcleaner.toml");
        load_config(&default_path).unwrap_or_default()
    };

    let (
        path,
        dry_run,
        quiet,
        max_depth,
        jobs,
        interactive,
        filter_str,
        skip_workspaces,
        log_level,
    ) = config.merge_with_cli(
        cli.path,
        cli.dry_run,
        cli.quiet,
        cli.max_depth,
        cli.jobs,
        cli.interactive,
        cli.filter,
        cli.skip_workspaces,
        cli.log_level,
    );

    init_logging(log_level, quiet);

    let filter = CrateFilter::from_options(filter_str.as_deref(), None).unwrap();
    let options = ScrubOptions {
        root: path,
        max_depth,
        dry_run,
        jobs,
        skip_workspaces,
        workspace_mode: cli.workspace_mode,
        filter,
        selected: None,
    };

    if cli.tui {
        return tui::run_tui(options).await;
    }

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

    // Discovery + filtering produced `crates`; the plan is the common
    // representation for check, dry-run, and real execution.
    let plan = build_clean_plan(&crates);

    if cli.check || dry_run {
        print_plan(&plan, dry_run && !cli.check);
        return Ok(());
    }

    let pb = Arc::new(cleaning_progress_bar(plan.crates.len()));
    let (report, reclaimed) = clean_selected(
        &options,
        plan,
        None,
        interactive,
        Some(pb),
    )
    .await?;

    report.print_summary();
    println!("{} {}", "Total cleaned space:".bold(), format_size(reclaimed));
    Ok(())
}

/// Print a check/dry-run listing from the plan. Does not spawn cargo.
fn print_plan(plan: &CleanPlan, dry_run: bool) {
    let title = if dry_run {
        "Dry run — Rust projects that would be cleaned:"
    } else {
        "Rust projects that would be cleaned:"
    };
    println!("{}", title.cyan());
    for crate_info in &plan.crates {
        println!("  {}", crate_info.path.display());
    }
    println!("{}", format!("Total: {}", plan.crates.len()).bold());
    println!(
        "{}",
        format!("Reclaimable: {}", format_size(plan.reclaimable_bytes())).bold()
    );
}
