// src/lib.rs

/* Public modules. */
pub mod auth;
pub mod chains;
pub mod cmd;
pub mod comm;
pub mod node;
pub mod ui;

use std::sync::OnceLock;

/**
 * Get Version
 *
 * Returns the crate version from `Cargo.toml`, embedded at compile time.
 */
pub fn get_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION.get_or_init(|| format!("v{}", env!("CARGO_PKG_VERSION")))
}

/**
 * Service Snapshot
 *
 * A single detected chain daemon on the host.
 */
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ServiceSnapshot {
    pub name: String,
    pub display_name: String,
    pub installed: bool,
    pub running: bool,
    pub version: String,
    pub pid: u32,
}
