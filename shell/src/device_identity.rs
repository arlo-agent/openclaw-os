//! Device identity — Ed25519 keypair for gateway authentication.
//!
//! Generates a persistent Ed25519 keypair on first run, stored at
//! `~/.openclaw-os/device-identity.json`. Used to sign connect challenges
//! so the gateway grants operator scopes.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};
use pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceIdentityFile {
    pub version: u32,
    #[serde(rename = "deviceId")]
    pub device_id: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
    #[serde(rename = "privateKeyPem")]
    pub private_key_pem: String,
}

pub struct DeviceIdentity {
    pub device_id: String,
    signing_key: SigningKey,
    raw_public_key: [u8; 32],
}

impl DeviceIdentity {
    /// Load an existing identity from disk, or generate and save a new one.
    pub fn load_or_create(path: &Path) -> Result<Self, String> {
        if path.exists() {
            Self::load(path)
        } else {
            Self::create_and_save(path)
        }
    }

    fn load(path: &Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read device identity: {}", e))?;
        let file: DeviceIdentityFile = serde_json::from_str(&data)
            .map_err(|e| format!("Failed to parse device identity: {}", e))?;

        let signing_key = SigningKey::from_pkcs8_pem(&file.private_key_pem)
            .map_err(|e| format!("Failed to decode private key PEM: {}", e))?;

        let raw_public_key = signing_key.verifying_key().to_bytes();

        // Verify device_id matches
        let computed_id = hex_sha256(&raw_public_key);
        if computed_id != file.device_id {
            return Err("Device ID mismatch — identity file may be corrupt".into());
        }

        Ok(Self {
            device_id: file.device_id,
            signing_key,
            raw_public_key,
        })
    }

    fn create_and_save(path: &Path) -> Result<Self, String> {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let raw_public_key = verifying_key.to_bytes();
        let device_id = hex_sha256(&raw_public_key);

        // Encode to PEM
        let private_pem = signing_key
            .to_pkcs8_pem(pkcs8::LineEnding::LF)
            .map_err(|e| format!("Failed to encode private key PEM: {}", e))?;
        let public_pem = verifying_key
            .to_public_key_pem(pkcs8::LineEnding::LF)
            .map_err(|e| format!("Failed to encode public key PEM: {}", e))?;

        let file = DeviceIdentityFile {
            version: 1,
            device_id: device_id.clone(),
            public_key_pem: public_pem,
            private_key_pem: private_pem.to_string(),
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize identity: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("Failed to write identity file: {}", e))?;

        eprintln!(
            "[device] Created new device identity: {} at {}",
            device_id,
            path.display()
        );

        Ok(Self {
            device_id,
            signing_key,
            raw_public_key,
        })
    }

    /// Sign a connect challenge and return (signature_b64url, public_key_b64url, signed_at_ms).
    pub fn sign_challenge(
        &self,
        nonce: &str,
        token: &str,
        client_id: &str,
        client_mode: &str,
        role: &str,
        scopes: &[&str],
        platform: &str,
    ) -> (String, String, u64) {
        let signed_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let scopes_csv = scopes.join(",");
        let device_family = "";

        let payload = format!(
            "v3|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.device_id,
            client_id,
            client_mode,
            role,
            scopes_csv,
            signed_at_ms,
            token,
            nonce,
            platform,
            device_family,
        );

        let signature = self.signing_key.sign(payload.as_bytes());
        let signature_b64url = URL_SAFE_NO_PAD.encode(signature.to_bytes());
        let public_key_b64url = URL_SAFE_NO_PAD.encode(self.raw_public_key);

        (signature_b64url, public_key_b64url, signed_at_ms)
    }
}

/// SHA-256 of raw bytes, returned as lowercase hex.
fn hex_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}
