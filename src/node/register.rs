// src/node/register.rs

/*!
 * Register — one-shot node registration with the control plane.
 *
 * Sends an initial telemetry snapshot and receives a `sessionid` that the
 * operator pastes into the web dashboard (or opens the printed link) to view
 * the node live.
 */

use serde::{Deserialize, Serialize};

use super::telemetry;

/// Server response to a registration request.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct RegistrationResponse {
    pub success: bool,
    pub result: SessionInfo,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct SessionInfo {
    pub sessionid: String,
    #[serde(rename = "hasAuth")]
    pub has_auth: bool,
    #[serde(rename = "createdAt")]
    pub created_at: u64, // seconds
}

/// Parse a registration response body.
pub fn parse_response(body: &str) -> Result<RegistrationResponse, serde_json::Error> {
    serde_json::from_str(body)
}

/// Build the registration payload (a telemetry snapshot tagged `register`).
pub fn build_payload() -> String {
    let mut t = telemetry::collect();
    t.method = "register".to_string();
    telemetry::to_json(&t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_success_response() {
        let body = r#"{"success":true,"result":{"sessionid":"abc123","hasAuth":true,"createdAt":1700000000}}"#;
        let r = parse_response(body).unwrap();
        assert!(r.success);
        assert_eq!(r.result.sessionid, "abc123");
        assert!(r.result.has_auth);
    }

    #[test]
    fn rejects_malformed() {
        assert!(parse_response("not json").is_err());
    }

    #[test]
    fn payload_is_register_method() {
        assert!(build_payload().contains("\"method\":\"register\""));
    }
}
