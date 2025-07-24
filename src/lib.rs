//! Core library for cargo-scrub: recursively cleaning Rust crates.

pub mod loglevel;
pub mod walker;
pub mod detector;
pub mod cleaner;
pub mod filter;
pub mod config;
pub mod report;
pub mod logging; 