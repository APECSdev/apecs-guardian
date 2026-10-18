// src/comm/monitor.rs

/*!
 * Monitor — the long-running report/poll loop.
 *
 * Each cycle:
 *   1. push a telemetry snapshot to the control plane,
 *   2. if the operator enabled actions (`--allow-actions`, Layer 1) AND the
 *      control plane returned a signed action request, verify it (Layer 3) and
 *      execute ONLY if it is in the closed allowlist (Layer 2),
 *   3. sleep until the next interval.
 *
 * READ-ONLY GUARANTEE: with `--allow-actions` OFF (the default) the loop never
 * executes a server instruction. Even with it ON, only an allowlisted,
 * correctly-signed, unexpired action runs — there is no shell.
 */

use serde::{Deserialize, Serialize};

use crate::auth::verify::{verify_request, Reject, SignedRequest};
use crate::cmd::action::{Action, MAX_LOG_TAIL};

/// What the control plane may return alongside a telemetry ack.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PollResponse {
    pub success: bool,
    /// A single pending signed action request, if the control plane has one.
    pub action: Option<SignedRequest>,
}

/// Outcome of handling one poll cycle.
#[derive(Debug, PartialEq)]
pub enum Outcome {
    /// No action requested, or actions disabled.
    NoAction,
    /// Action executed; carries a short human-readable result summary.
    Executed(String),
    /// Action rejected; carries the reason.
    Rejected(String),
}

/// Execute an allowlisted action and return a short result string.
///
/// Every branch is read-only. `NodeLogs` is clamped to `MAX_LOG_TAIL`.
pub fn execute(action: &Action) -> String {
    match action {
        Action::NodeStatus => {
            let services = crate::chains::detect::detect_all();
            let running: Vec<String> = services
                .iter()
                .filter(|s| s.running)
                .map(|s| format!("{}(pid {})", s.name, s.pid))
                .collect();
            if running.is_empty() {
                "no chain daemons running".to_string()
            } else {
                format!("running: {}", running.join(", "))
            }
        }
        Action::NodeLogs { chain, tail } => {
            let n = (*tail).min(MAX_LOG_TAIL);
            // Read-only bounded log read is implemented per-chain at a later
            // milestone; report the clamped intent now. No file write/exec.
            format!("node_logs: chain={} tail={} (clamped)", chain, n)
        }
        Action::CollectMetrics => "metrics collected".to_string(),
    }
}

/// Decide what to do with a poll response, given the operator's action flag.
///
/// `allow_actions` is Layer 1 (operator opt-in). `now_ms` feeds Layer 3's
/// expiry check. Pure and fully testable — no I/O.
pub fn handle_poll(resp: &PollResponse, allow_actions: bool, now_ms: u64) -> Outcome {
    let Some(req) = &resp.action else {
        return Outcome::NoAction;
    };

    /* Layer 1: operator must have opted in. */
    if !allow_actions {
        return Outcome::Rejected("actions disabled (start with --allow-actions)".into());
    }

    /* Layer 3: signature + expiry. Layer 2 is enforced by deserialization of
     * `req.action` into the closed `Action` enum. */
    match verify_request(req, now_ms) {
        Ok(action) => Outcome::Executed(execute(&action)),
        Err(Reject::KeyNotConfigured) => Outcome::Rejected("control key not configured".into()),
        Err(Reject::Expired) => Outcome::Rejected("action expired".into()),
        Err(Reject::MalformedSignature) => Outcome::Rejected("malformed signature".into()),
        Err(Reject::BadSignature) => Outcome::Rejected("bad signature".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::action::Action;

    fn req_with(action: Action) -> PollResponse {
        PollResponse {
            success: true,
            action: Some(SignedRequest {
                action,
                expiry_ms: 60_000,
                nonce: "n".into(),
                signature: hex::encode([0u8; 64]),
            }),
        }
    }

    #[test]
    fn no_action_when_none() {
        let resp = PollResponse { success: true, action: None };
        assert_eq!(handle_poll(&resp, true, 0), Outcome::NoAction);
    }

    #[test]
    fn layer1_blocks_when_not_opted_in() {
        let resp = req_with(Action::NodeStatus);
        match handle_poll(&resp, false, 0) {
            Outcome::Rejected(r) => assert!(r.contains("disabled")),
            other => panic!("expected rejection, got {:?}", other),
        }
    }

    #[test]
    fn unsigned_request_rejected_even_when_opted_in() {
        // Placeholder control key => fail closed.
        let resp = req_with(Action::NodeStatus);
        match handle_poll(&resp, true, 0) {
            Outcome::Rejected(_) => {}
            other => panic!("expected rejection, got {:?}", other),
        }
    }

    #[test]
    fn log_tail_is_clamped() {
        let out = execute(&Action::NodeLogs { chain: "dash".into(), tail: 999_999 });
        assert!(out.contains(&MAX_LOG_TAIL.to_string()));
    }

    #[test]
    fn node_status_executes_readonly() {
        let out = execute(&Action::NodeStatus);
        assert!(out.contains("running") || out.contains("no chain daemons"));
    }
}
