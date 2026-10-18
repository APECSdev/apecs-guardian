// src/auth/verify.rs

/*!
 * Verify — Layer 3. Authenticate a signed action request from the control
 * plane before it is allowed to execute.
 *
 * A request is honoured ONLY if ALL of the following hold:
 *   1. the action deserializes into the closed `Action` allowlist (Layer 2),
 *   2. the Ed25519 signature verifies against the baked-in control public key,
 *   3. the request has not expired,
 *   4. the operator enabled remote actions (`--allow-actions`) — Layer 1.
 *
 * A request that fails ANY check is rejected. A compromised API that does not
 * also hold the offline control private key cannot forge a valid signature.
 */

use ed25519_dalek::{Signature, VerifyingKey};

use crate::cmd::action::{signing_payload, Action};

/// APECS control-plane PUBLIC key (hex), baked in at build time.
///
/// Generated once via `apecs keygen`; the matching PRIVATE key is held only in
/// the APECS secret store (`/Workspace/apecsdev/.secrets/guardian-control-plane.json`)
/// and is used by the control plane to sign action requests. Rotating the key
/// means updating this constant and rebuilding the daemon.
pub const CONTROL_PUBKEY_HEX: &str =
    "50f7ccae49847f5578b0ea2ffe08eeb1f5504e472c016d270908cffdcd153491";

/// Maximum accepted clock skew / request lifetime: 5 minutes.
pub const MAX_AGE_MS: u64 = 5 * 60 * 1000;

/// A signed action request delivered by the control plane.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignedRequest {
    pub action: Action,
    /// Unix epoch milliseconds after which the request is void.
    pub expiry_ms: u64,
    /// One-time value bound into the signature (replay resistance).
    pub nonce: String,
    /// Hex Ed25519 signature over `signing_payload(action, expiry_ms, nonce)`.
    pub signature: String,
}

#[derive(Debug, PartialEq)]
pub enum Reject {
    /// Control key not configured (still the placeholder).
    KeyNotConfigured,
    /// Signature field was not valid hex / not a valid signature.
    MalformedSignature,
    /// Signature did not verify against the control public key.
    BadSignature,
    /// Request is past its expiry.
    Expired,
}

/// Load the baked-in control public key.
fn control_pubkey() -> Result<VerifyingKey, Reject> {
    if CONTROL_PUBKEY_HEX.bytes().all(|b| b == b'0') {
        return Err(Reject::KeyNotConfigured);
    }
    let raw = hex::decode(CONTROL_PUBKEY_HEX).map_err(|_| Reject::KeyNotConfigured)?;
    let bytes: [u8; 32] = raw.try_into().map_err(|_| Reject::KeyNotConfigured)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| Reject::KeyNotConfigured)
}

/// Verify a signed request against the baked-in control public key.
/// Returns the validated `Action` on success.
///
/// `now_ms` is injected for deterministic testing.
pub fn verify_request(req: &SignedRequest, now_ms: u64) -> Result<Action, Reject> {
    let pubkey = control_pubkey()?;
    verify_with_key(req, now_ms, &pubkey)
}

/// Verify a signed request against an explicit public key. This is the full
/// verification path; `verify_request` is this logic bound to the baked-in
/// control key. Split out so tests can exercise the identical path against a
/// freshly generated keypair — the production private key is never in source.
fn verify_with_key(req: &SignedRequest, now_ms: u64, pubkey: &VerifyingKey) -> Result<Action, Reject> {
    /* Expiry check first — cheap and key-independent. */
    if now_ms > req.expiry_ms || req.expiry_ms.saturating_sub(now_ms) > MAX_AGE_MS {
        return Err(Reject::Expired);
    }

    let sig_bytes = hex::decode(&req.signature).map_err(|_| Reject::MalformedSignature)?;
    let sig_arr: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| Reject::MalformedSignature)?;
    let signature = Signature::from_bytes(&sig_arr);

    let payload = signing_payload(&req.action, req.expiry_ms, &req.nonce);

    pubkey
        .verify_strict(&payload, &signature)
        .map_err(|_| Reject::BadSignature)?;

    Ok(req.action.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    /// Generate a THROWAWAY keypair for the test. The production control-plane
    /// private key is NEVER in source — tests prove the sign/verify path against
    /// a key they generate themselves, which exercises identical logic.
    fn fresh_keypair() -> (ed25519_dalek::SigningKey, VerifyingKey) {
        let sk = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();
        (sk, vk)
    }

    /// Build a request signed by the given (throwaway) key.
    fn signed_by(sk: &ed25519_dalek::SigningKey, action: &Action, expiry: u64, nonce: &str) -> SignedRequest {
        let payload = signing_payload(action, expiry, nonce);
        let sig = sk.sign(&payload);
        SignedRequest {
            action: action.clone(),
            expiry_ms: expiry,
            nonce: nonce.to_string(),
            signature: hex::encode(sig.to_bytes()),
        }
    }

    #[test]
    fn baked_in_key_is_not_the_placeholder() {
        assert!(!CONTROL_PUBKEY_HEX.bytes().all(|b| b == b'0'));
        assert_eq!(CONTROL_PUBKEY_HEX.len(), 64);
        /* The baked-in key must parse as a valid Ed25519 verifying key. */
        assert!(control_pubkey().is_ok());
    }

    #[test]
    fn correctly_signed_request_verifies_end_to_end() {
        // Full path: sign with a key, verify against its public half.
        let (sk, vk) = fresh_keypair();
        let req = signed_by(&sk, &Action::NodeStatus, 60_000, "nonce-1");
        assert_eq!(verify_with_key(&req, 5_000, &vk), Ok(Action::NodeStatus));
    }

    #[test]
    fn wrong_key_is_rejected() {
        // Signed by one key, verified against a DIFFERENT key: must fail.
        let (sk, _vk) = fresh_keypair();
        let (_sk2, vk2) = fresh_keypair();
        let req = signed_by(&sk, &Action::NodeStatus, 60_000, "n");
        assert_eq!(verify_with_key(&req, 5_000, &vk2), Err(Reject::BadSignature));
    }

    #[test]
    fn expired_is_rejected_before_key_check() {
        let (sk, vk) = fresh_keypair();
        let req = signed_by(&sk, &Action::NodeStatus, 1_000, "n");
        assert_eq!(verify_with_key(&req, 999_999, &vk), Err(Reject::Expired));
    }

    #[test]
    fn tampered_payload_fails() {
        // Sign, then change the nonce: the signature no longer matches.
        let (sk, vk) = fresh_keypair();
        let mut req = signed_by(&sk, &Action::NodeStatus, 60_000, "nonce-1");
        req.nonce = "nonce-2".into();
        assert_eq!(verify_with_key(&req, 5_000, &vk), Err(Reject::BadSignature));
    }

    #[test]
    fn malformed_signature_rejected() {
        let (sk, vk) = fresh_keypair();
        let mut req = signed_by(&sk, &Action::NodeStatus, 60_000, "n");
        req.signature = "zz".into();
        assert_eq!(verify_with_key(&req, 5_000, &vk), Err(Reject::MalformedSignature));
    }

    #[test]
    fn node_logs_action_round_trips() {
        // The signed payload binds the full action (chain + tail), not just the op.
        let (sk, vk) = fresh_keypair();
        let action = Action::NodeLogs { chain: "bitcoin".into(), tail: 100 };
        let req = signed_by(&sk, &action, 60_000, "n");
        assert_eq!(verify_with_key(&req, 5_000, &vk), Ok(action));
    }
}
