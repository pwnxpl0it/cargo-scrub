//! Logging level enum for cargo-scrub.

use clap::ValueEnum;

#[derive(ValueEnum, Debug, Clone)]
pub enum LogLevel {
    Error,
    Info,
    Debug,
    Silent,
} 