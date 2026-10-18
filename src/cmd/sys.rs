// src/cmd/sys.rs

/*!
 * Sys — read-only host probes. Every function shells out to a standard
 * read-only utility or reads `/proc`/`/sys`. None of them mutate the host.
 *
 * On non-Linux platforms the probes degrade gracefully to empty/zero values;
 * the daemon still runs and reports what it can.
 */

use std::process::Command;

/// Run a read-only command and return trimmed stdout, or None on any failure.
fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Read a single-line value from a sysfs/procfs file.
fn read_first_line(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content.lines().next().map(|l| l.trim().to_string())
}

pub fn get_hostname() -> Option<String> {
    run("hostname", &[]).or_else(|| read_first_line("/proc/sys/kernel/hostname"))
}

pub fn get_os_pretty() -> Option<String> {
    // PRETTY_NAME from /etc/os-release, falling back to `uname -s`.
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                return Some(rest.trim_matches('"').to_string());
            }
        }
    }
    run("uname", &["-s"])
}

pub fn get_arch() -> Option<String> {
    run("uname", &["-m"])
}

pub fn get_kernel() -> Option<String> {
    run("uname", &["-r"])
}

/// Stable machine identifier (systemd machine-id; empty on platforms without it).
pub fn get_machine_id() -> Option<String> {
    read_first_line("/etc/machine-id").or_else(|| read_first_line("/var/lib/dbus/machine-id"))
}

pub fn get_cpu_model() -> Option<String> {
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("model name") {
                if let Some((_, v)) = rest.split_once(':') {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}

pub fn get_cpu_cores() -> Option<u32> {
    let content = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let cores = content.matches("processor\t:").count() as u32;
    if cores > 0 { Some(cores) } else { None }
}

pub fn get_mem_total_mb() -> Option<u64> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
            return Some(kb / 1024);
        }
    }
    None
}

pub fn get_uptime() -> Option<String> {
    let secs: f64 = read_first_line("/proc/uptime")?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let s = secs as u64;
    Some(format!("{}d {}h {}m", s / 86400, (s % 86400) / 3600, (s % 3600) / 60))
}

/// 1/5/15-minute load averages.
pub fn get_load_avg() -> Option<(f64, f64, f64)> {
    let line = read_first_line("/proc/loadavg")?;
    let mut it = line.split_whitespace();
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next()?.parse().ok()?;
    Some((a, b, c))
}

/// Used-disk percentage of the root filesystem (0–100).
pub fn get_disk_used_pct() -> Option<u8> {
    let out = run("df", &["--output=pcent", "/"])?;
    let line = out.lines().nth(1)?; // skip header
    line.trim().trim_end_matches('%').parse().ok()
}

/// Total size of the root filesystem in GB.
pub fn get_disk_total_gb() -> Option<u64> {
    let out = run("df", &["--block-size=1G", "--output=size", "/"])?;
    let line = out.lines().nth(1)?;
    line.trim().trim_end_matches('G').trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_is_some_on_linux() {
        // Guarded: only asserts on hosts where `hostname`/`/proc` exist.
        if cfg!(target_os = "linux") {
            assert!(get_hostname().is_some());
        }
    }

    #[test]
    fn mem_total_positive_on_linux() {
        if cfg!(target_os = "linux") {
            assert!(get_mem_total_mb().unwrap_or(0) > 0);
        }
    }

    #[test]
    fn load_avg_parses() {
        if cfg!(target_os = "linux") {
            let (a, _, _) = get_load_avg().unwrap();
            assert!(a >= 0.0);
        }
    }
}
