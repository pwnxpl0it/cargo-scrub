//! Filtering logic for skipping or including crates.

use regex::Regex;
use std::path::Path;

pub struct CrateFilter {
    pub regex: Option<Regex>,
    pub path_substr: Option<String>,
}

impl CrateFilter {
    /// Returns true if the given path matches the filter.
    pub fn matches(&self, path: &Path) -> bool {
        // TODO: Implement filter logic
        unimplemented!()
    }
} 