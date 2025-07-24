//! Logging setup for cargo-scrub.

use crate::cli::LogLevel;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

/// Initialize logging based on log level and quiet flag.
pub fn init_logging(level: LogLevel, quiet: bool) {
    let mut builder = Builder::new();
    let filter = match (level, quiet) {
        (_, true) => LevelFilter::Error,
        (LogLevel::Error, _) => LevelFilter::Error,
        (LogLevel::Info, _) => LevelFilter::Info,
        (LogLevel::Debug, _) => LevelFilter::Debug,
        (LogLevel::Silent, _) => LevelFilter::Off,
    };
    builder.filter_level(filter);
    builder.format(|buf, record| {
        writeln!(buf, "[{}] {}", record.level(), record.args())
    });
    let _ = builder.try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_mapping() {
        assert_eq!(map_log_level(LogLevel::Info, false), LevelFilter::Info);
        assert_eq!(map_log_level(LogLevel::Debug, false), LevelFilter::Debug);
        assert_eq!(map_log_level(LogLevel::Error, false), LevelFilter::Error);
        assert_eq!(map_log_level(LogLevel::Silent, false), LevelFilter::Off);
        assert_eq!(map_log_level(LogLevel::Info, true), LevelFilter::Error);
    }

    fn map_log_level(level: LogLevel, quiet: bool) -> LevelFilter {
        match (level, quiet) {
            (_, true) => LevelFilter::Error,
            (LogLevel::Error, _) => LevelFilter::Error,
            (LogLevel::Info, _) => LevelFilter::Info,
            (LogLevel::Debug, _) => LevelFilter::Debug,
            (LogLevel::Silent, _) => LevelFilter::Off,
        }
    }
} 