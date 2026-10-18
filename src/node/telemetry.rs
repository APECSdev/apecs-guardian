// src/node/telemetry.rs

/*!
 * Telemetry — the read-only snapshot the daemon reports to the control plane.
 * Pure assembly: gathers host + chain state into a serializable struct. No
 * network I/O happens here (that lives in `comm::monitor`).
 */

use serde::{Deserialize, Serialize};

use crate::chains::detect;
use crate::cmd::sys;
use crate::ServiceSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Telemetry {
    pub method: String,

    /* Identity */
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub kernel: String,
    pub machine_id: String,

    /* Hardware */
    pub cpu_model: String,
    pub cpu_cores: u32,
    pub mem_total_mb: u64,
    pub disk_total_gb: u64,

    /* Runtime */
    pub uptime: String,
    pub disk_used_pct: u8,
    pub load_avg: (f64, f64, f64),

    /* Detected chain daemons */
    pub services: Vec<ServiceSnapshot>,
}

/// Assemble a telemetry snapshot from live host probes.
pub fn collect() -> Telemetry {
    Telemetry {
        method: "telemetry".to_string(),
        hostname: sys::get_hostname().unwrap_or_default(),
        os: sys::get_os_pretty().unwrap_or_default(),
        arch: sys::get_arch().unwrap_or_default(),
        kernel: sys::get_kernel().unwrap_or_default(),
        machine_id: sys::get_machine_id().unwrap_or_default(),
        cpu_model: sys::get_cpu_model().unwrap_or_default(),
        cpu_cores: sys::get_cpu_cores().unwrap_or(0),
        mem_total_mb: sys::get_mem_total_mb().unwrap_or(0),
        disk_total_gb: sys::get_disk_total_gb().unwrap_or(0),
        uptime: sys::get_uptime().unwrap_or_default(),
        disk_used_pct: sys::get_disk_used_pct().unwrap_or(0),
        load_avg: sys::get_load_avg().unwrap_or((0.0, 0.0, 0.0)),
        services: detect::detect_all(),
    }
}

/// Serialize a telemetry snapshot to JSON.
pub fn to_json(t: &Telemetry) -> String {
    serde_json::to_string(t).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_serializes() {
        let t = collect();
        let json = to_json(&t);
        assert!(json.contains("\"method\":\"telemetry\""));
        // All six supported chains are always reported.
        assert_eq!(t.services.len(), 6);
    }
}
