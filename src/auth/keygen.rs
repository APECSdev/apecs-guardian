// src/auth/keygen.rs

/*!
 * Keygen — one-time control-plane key generation.
 *
 * Run once by the operator of the APECS control plane:
 *
 * ```text
 * apecs keygen
 * ```
 *
 * Prints a fresh Ed25519 keypair. The PUBLIC key is baked into the daemon
 * (see `auth::verify::CONTROL_PUBKEY_HEX`) so nodes can authenticate signed
 * action requests. The PRIVATE key is held ONLY by the control plane and is
 * used to sign each action request. It is never shipped and never committed.
 */

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

/// A freshly generated control-plane keypair (hex-encoded).
pub struct ControlKeys {
    pub public_hex: String,
    pub private_hex: String,
}

/// Generate a new Ed25519 keypair from the OS CSPRNG.
pub fn generate() -> ControlKeys {
    let signing = SigningKey::generate(&mut OsRng);
    let verify = signing.verifying_key();

    ControlKeys {
        public_hex: hex::encode(verify.to_bytes()),
        private_hex: hex::encode(signing.to_bytes()),
    }
}

/// Print a keypair with explicit handling instructions.
pub fn print_instructions(keys: &ControlKeys) {
    println!("\n  APECS Guardian — control-plane keypair\n");
    println!("  PUBLIC key  (bake into daemon, auth::verify::CONTROL_PUBKEY_HEX):");
    println!("    {}\n", keys.public_hex);
    println!("  PRIVATE key (KEEP SECRET — control plane signs actions with it):");
    println!("    {}\n", keys.private_hex);
    println!("  The private key is shown ONCE and is NOT stored. Record it in the");
    println!("  APECS secret store. Never commit it. Rotating it means rebuilding");
    println!("  the daemon with the new public key.\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_32_byte_keys() {
        let k = generate();
        assert_eq!(k.public_hex.len(), 64); // 32 bytes as hex
        assert_eq!(k.private_hex.len(), 64);
        assert!(hex::decode(&k.public_hex).is_ok());
        assert!(hex::decode(&k.private_hex).is_ok());
    }

    #[test]
    fn keys_match() {
        use ed25519_dalek::Signer;
        let k = generate();
        let sk_bytes: [u8; 32] = hex::decode(&k.private_hex).unwrap().try_into().unwrap();
        let sk = SigningKey::from_bytes(&sk_bytes);
        let sig = sk.sign(b"test");
        let vk_bytes: [u8; 32] = hex::decode(&k.public_hex).unwrap().try_into().unwrap();
        let vk = ed25519_dalek::VerifyingKey::from_bytes(&vk_bytes).unwrap();
        assert!(vk.verify_strict(b"test", &sig).is_ok());
    }
}
