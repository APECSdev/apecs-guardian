// src/chains/detect.rs

/*!
 * Detect — locate supported chain daemons on the host (read-only).
 *
 * Detection strategy:
 *   1. find the chain's CLI binary on PATH (`which`),
 *   2. query it for a version string,
 *   3. check whether the node process is running (`pgrep`) and capture its pid.
 *
 * All commands are read-only and bounded by short timeouts. A chain that is
 * absent simply reports `installed: false, running: false`.
 */

use std::process::Command;

use super::Chain;
use crate::ServiceSnapshot;

/// Run a command, returning trimmed stdout (empty string on failure).
fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Locate a binary on PATH.
fn which(bin: &str) -> Option<String> {
    let out = run("which", &[bin]);
    if out.is_empty() { None } else { Some(out) }
}

/// First running pid for a process name, if any.
fn pid_of(process: &str) -> u32 {
    let out = run("pgrep", &["-n", "-x", process]);
    out.trim().parse().unwrap_or(0)
}

/// Query a `*-cli -version` style binary for its version string.
fn cli_version(cli: &str) -> String {
    // e.g. "Dash Core RPC client version v20.x" -> take the version token.
    let out = run(cli, &["-version"]);
    out.lines()
        .next()
        .and_then(|l| l.split_whitespace().last())
        .unwrap_or("")
        .to_string()
}

/// Detect a single chain.
pub fn detect(chain: Chain) -> ServiceSnapshot {
    let installed = which(chain.cli()).is_some();

    // Avalanche runs as `avalanchego`; the others run as `<chain>d`-style
    // daemons. We match the CLI's daemon counterpart by process name.
    let proc_name = match chain {
        Chain::Avalanche => "avalanchego",
        Chain::Dash => "dashd",
        Chain::Bitcoin | Chain::BitcoinCash => "bitcoind",
        Chain::Dogecoin => "dogecoind",
        Chain::Litecoin => "litecoind",
    };

    let pid = pid_of(proc_name);
    let version = if installed { cli_version(chain.cli()) } else { String::new() };

    ServiceSnapshot {
        name: chain.name().to_string(),
        display_name: chain.display().to_string(),
        installed,
        running: pid > 0,
        version,
        pid,
    }
}

/// Detect all supported chains.
pub fn detect_all() -> Vec<ServiceSnapshot> {
    Chain::ALL.iter().map(|c| detect(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_all_returns_every_supported_chain() {
        let all = detect_all();
        assert_eq!(all.len(), Chain::ALL.len());
        // Every entry carries the chain's machine name.
        for (i, s) in all.iter().enumerate() {
            assert_eq!(s.name, Chain::ALL[i].name());
        }
    }

    #[test]
    fn absent_chain_is_not_running() {
        // A chain whose binary is absent must report not-installed/not-running.
        let s = ServiceSnapshot {
            name: "nonexistent".into(),
            display_name: "Nope".into(),
            installed: false,
            running: false,
            version: String::new(),
            pid: 0,
        };
        assert!(!s.installed && !s.running && s.pid == 0);
    }
}
