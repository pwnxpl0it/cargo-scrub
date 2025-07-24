//! Summary reporting for cargo-scrub.

use std::time::Duration;
use std::path::PathBuf;

pub struct SummaryReport {
    pub cleaned: usize,
    pub skipped: usize,
    pub errors: usize,
    pub total: usize,
    pub duration: Duration,
    pub details: Vec<(PathBuf, bool, Option<String>)>, // (path, success, error)
}

impl SummaryReport {
    /// Print a summary report to the console.
    pub fn print_summary(&self) {
        // TODO: Implement pretty summary output
        unimplemented!()
    }
} 