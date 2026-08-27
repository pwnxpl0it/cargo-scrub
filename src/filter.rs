//! Filtering logic for skipping or including crates.

use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CrateFilter {
    pub regex: Option<Regex>,
    pub path_substr: Option<String>,
}

impl CrateFilter {
    /// Returns true if the given path matches the filter.
    pub fn matches(&self, path: &Path) -> bool {
        if self.regex.is_none() && self.path_substr.is_none() {
            return true;
        }
        let path_str = path.to_string_lossy();
        if let Some(ref re) = self.regex {
            if re.is_match(&path_str) {
                return true;
            }
        }
        if let Some(ref substr) = self.path_substr {
            if path_str.contains(substr) {
                return true;
            }
        }
        false
    }

    /// Create a filter from regex string or substring.
    pub fn from_options(regex: Option<&str>, path_substr: Option<&str>) -> Result<Self, regex::Error> {
        Ok(Self {
            regex: match regex { Some(r) => Some(Regex::new(r)?), None => None },
            path_substr: path_substr.map(|s| s.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_regex_filter() {
        let filter = CrateFilter::from_options(Some("foo"), None).unwrap();
        assert!(filter.matches(&PathBuf::from("/some/foo/bar")));
        assert!(!filter.matches(&PathBuf::from("/some/bar/baz")));
    }

    #[test]
    fn test_substr_filter() {
        let filter = CrateFilter::from_options(None, Some("bar")).unwrap();
        assert!(filter.matches(&PathBuf::from("/some/foo/bar")));
        assert!(!filter.matches(&PathBuf::from("/some/foo/baz")));
    }

    #[test]
    fn test_both_filters() {
        let filter = CrateFilter::from_options(Some("foo"), Some("baz")).unwrap();
        assert!(filter.matches(&PathBuf::from("/foo/abc")));
        assert!(filter.matches(&PathBuf::from("/abc/baz")));
        assert!(!filter.matches(&PathBuf::from("/abc/def")));
    }
} 