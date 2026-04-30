//! Capability package signature verification.

use crate::{DurabilityError, Result};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

const SIGNATURE_SCHEMA: &str = "convergio.capability.signature.v1";

/// Trusted Ed25519 public key allowed to sign capability packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedCapabilityKey {
    /// Stable key identifier from Convergio trust metadata.
    pub key_id: String,
    /// Ed25519 public key as 32-byte lowercase or uppercase hex.
    pub public_key: String,
}

/// Signature verification request for one capability package descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySignatureRequest {
    /// Capability name.
    pub name: String,
    /// Capability package version.
    pub version: String,
    /// Package checksum in `sha256:<64 hex chars>` form.
    pub checksum: String,
    /// Parsed capability manifest.
    pub manifest: Value,
    /// Detached Ed25519 signature as 64-byte hex.
    pub signature: String,
    /// Trusted keys configured by core or explicitly pinned by a local user.
    #[serde(default)]
    pub trusted_keys: Vec<TrustedCapabilityKey>,
}

/// Successful capability signature verification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySignatureVerification {
    /// Capability name.
    pub name: String,
    /// Capability package version.
    pub version: String,
    /// Package checksum that was covered by the signature payload.
    pub checksum: String,
    /// Trusted key that verified the signature.
    pub key_id: String,
    /// SHA-256 digest of the manifest JSON bytes.
    pub manifest_sha256: String,
    /// SHA-256 digest of the canonical signature payload.
    pub payload_sha256: String,
}

/// Build the canonical payload bytes that capability signatures cover.
pub fn capability_signature_payload(input: &CapabilitySignatureRequest) -> Result<Vec<u8>> {
    validate_descriptor(input)?;
    let manifest_sha = manifest_sha256(&input.manifest)?;
    Ok(format!(
        "{SIGNATURE_SCHEMA}\nname={}\nversion={}\nchecksum={}\nmanifest_sha256={manifest_sha}\n",
        input.name, input.version, input.checksum
    )
    .into_bytes())
}

/// Verify a detached capability package signature against trusted keys.
pub fn verify_capability_signature(
    input: &CapabilitySignatureRequest,
) -> Result<CapabilitySignatureVerification> {
    if input.trusted_keys.is_empty() {
        return invalid("at least one trusted capability key is required");
    }
    let payload = capability_signature_payload(input)?;
    let signature = parse_signature(&input.signature)?;
    for key in &input.trusted_keys {
        validate_key_id(&key.key_id)?;
        let verifying_key = parse_public_key(&key.public_key)?;
        if verifying_key.verify_strict(&payload, &signature).is_ok() {
            return Ok(CapabilitySignatureVerification {
                name: input.name.clone(),
                version: input.version.clone(),
                checksum: input.checksum.clone(),
                key_id: key.key_id.clone(),
                manifest_sha256: manifest_sha256(&input.manifest)?,
                payload_sha256: sha256_hex(&payload),
            });
        }
    }
    invalid("capability signature did not verify against trusted keys")
}

fn validate_descriptor(input: &CapabilitySignatureRequest) -> Result<()> {
    validate_name(&input.name)?;
    validate_no_newline("version", &input.version)?;
    validate_checksum(&input.checksum)?;
    Ok(())
}

fn validate_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_');
    if valid {
        Ok(())
    } else {
        invalid("capability name must be lowercase ascii, digit, '-' or '_'")
    }
}

fn validate_no_newline(field: &str, value: &str) -> Result<()> {
    if value.is_empty() || value.contains('\n') || value.contains('\r') {
        invalid(format!("{field} must be non-empty and single-line"))
    } else {
        Ok(())
    }
}

fn validate_checksum(checksum: &str) -> Result<()> {
    let Some(hex) = checksum.strip_prefix("sha256:") else {
        return invalid("checksum must start with sha256:");
    };
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        invalid("checksum must be sha256:<64 hex chars>")
    }
}

fn validate_key_id(key_id: &str) -> Result<()> {
    validate_no_newline("key_id", key_id)
}

fn parse_public_key(hex_value: &str) -> Result<VerifyingKey> {
    let bytes = decode_fixed_hex::<32>(hex_value, "public key")?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| invalid_err("invalid ed25519 public key"))
}

fn parse_signature(hex_value: &str) -> Result<Signature> {
    if hex_value.is_empty() {
        return invalid("capability signature is required");
    }
    let bytes = decode_fixed_hex::<64>(hex_value, "signature")?;
    Ok(Signature::from_bytes(&bytes))
}

fn decode_fixed_hex<const N: usize>(value: &str, label: &str) -> Result<[u8; N]> {
    let bytes = hex::decode(value).map_err(|_| invalid_err(format!("{label} must be hex")))?;
    bytes
        .try_into()
        .map_err(|_| invalid_err(format!("{label} must be {N} bytes")))
}

fn manifest_sha256(manifest: &Value) -> Result<String> {
    let bytes = serde_json::to_vec(manifest)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn invalid<T>(reason: impl Into<String>) -> Result<T> {
    Err(invalid_err(reason))
}

fn invalid_err(reason: impl Into<String>) -> DurabilityError {
    DurabilityError::InvalidCapability {
        reason: reason.into(),
    }
}
