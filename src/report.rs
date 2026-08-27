//! Summary reporting for cargo-scrub.

use std::time::Duration;
use std::path::PathBuf;
use colored::*;

#[derive(Debug, Clone)]
pub struct SummaryReport {
    pub cleaned: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total: usize,
    pub duration: Duration,
    pub details: Vec<(PathBuf, bool, Option<String>)>, // (path, success, error)
}

/// Format bytes as a human-readable string.
pub fn format_size(bytes: u64) -> String {
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

impl SummaryReport {
    /// Print a summary report to the console.
    pub fn print_summary(&self) {
        println!("\n{}", "Cargo Scrub Summary".bold().underline());
        println!("{}: {}", "Total".cyan(), self.total);
        println!("{}: {}", "Cleaned".green(), self.cleaned);
        println!("{}: {}", "Skipped".yellow(), self.skipped);
        println!("{}: {}", "Errors".red(), self.errors);
        println!("{}: {:.2?}", "Duration".blue(), self.duration);
        println!("\n{}", "Per-crate results:".bold());
        for (path, success, error) in &self.details {
            if *success {
                println!("  {} {}", "✔".green(), path.display());
            } else if let Some(err) = error {
                println!("  {} {}: {}", "✗".red(), path.display(), err.red());
            } else {
                println!("  {} {}", "-".yellow(), path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_print_summary() {
        let report = SummaryReport {
            cleaned: 2,
            skipped: 1,
            errors: 1,
            total: 4,
            duration: Duration::from_secs(1),
            details: vec![
                (PathBuf::from("/a"), true, None),
                (PathBuf::from("/b"), false, Some("fail".to_string())),
                (PathBuf::from("/c"), false, None),
                (PathBuf::from("/d"), true, None),
            ],
        };
        report.print_summary();
    }
} 