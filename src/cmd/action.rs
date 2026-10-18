// src/cmd/action.rs

/*!
 * Action — the CLOSED allowlist of remote-triggerable operations (Layer 2).
 *
 * SECURITY INVARIANT: this is a compile-time enum, NOT a free-text command
 * string. The server can only ever *name* one of these variants; there is no
 * code path that turns a server payload into an arbitrary shell command.
 * Anything that does not deserialize into a known variant is rejected before
 * execution. There is deliberately NO `Install`, `Build`, or `Shell` variant.
 */

use serde::{Deserialize, Serialize};

/// The complete set of operations the control plane may request.
///
/// Every variant is READ-ONLY with respect to the host. Adding a new variant
/// is a deliberate, reviewed code change — it can never be smuggled in via a
/// network message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Action {
    /// Report the status of each detected chain daemon.
    NodeStatus,

    /// Read the tail of a chain daemon's log, bounded to `tail` lines.
    /// `tail` is clamped to a hard maximum at execution time.
    NodeLogs { chain: String, tail: u32 },

    /// Trigger an immediate telemetry push out of schedule.
    CollectMetrics,
}

/// Hard cap on log lines returned by `NodeLogs`, regardless of request.
pub const MAX_LOG_TAIL: u32 = 500;

/// Canonical byte encoding that is signed/verified (Layer 3).
///
/// Encoding: `"{op}|{expiry_unix_ms}|{nonce}"`. Binding the expiry and nonce
/// into the signed payload prevents replay of a captured signature.
pub fn signing_payload(action: &Action, expiry_ms: u64, nonce: &str) -> Vec<u8> {
    let op = serde_json::to_string(action).unwrap_or_default();
    format!("{}|{}|{}", op, expiry_ms, nonce).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_op() {
        let bad = r#"{"op":"shell","cmd":"rm -rf /"}"#;
        assert!(serde_json::from_str::<Action>(bad).is_err());
    }

    #[test]
    fn rejects_free_text() {
        assert!(serde_json::from_str::<Action>(r#""install avax""#).is_err());
    }

    #[test]
    fn round_trips_known_ops() {
        let a: Action = serde_json::from_str(r#"{"op":"node_status"}"#).unwrap();
        assert_eq!(a, Action::NodeStatus);

        let b: Action =
            serde_json::from_str(r#"{"op":"node_logs","chain":"dash","tail":50}"#).unwrap();
        assert_eq!(b, Action::NodeLogs { chain: "dash".into(), tail: 50 });

        let c: Action = serde_json::from_str(r#"{"op":"collect_metrics"}"#).unwrap();
        assert_eq!(c, Action::CollectMetrics);
    }

    #[test]
    fn payload_binds_expiry_and_nonce() {
        let p1 = signing_payload(&Action::NodeStatus, 1000, "abc");
        let p2 = signing_payload(&Action::NodeStatus, 1001, "abc");
        let p3 = signing_payload(&Action::NodeStatus, 1000, "abd");
        assert_ne!(p1, p2);
        assert_ne!(p1, p3);
    }
}
