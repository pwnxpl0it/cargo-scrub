//! Logging level enum for cargo-scrub.

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Info,
    Debug,
    Silent,
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "silent" => Ok(LogLevel::Silent),
            other => Err(format!("unknown log level: {}", other)),
        }
    }
} 