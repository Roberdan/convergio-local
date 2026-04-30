//! `cvg capability` — local capability registry diagnostics.

use super::{Client, OutputMode};
use anyhow::Result;
use clap::Subcommand;
use convergio_i18n::Bundle;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Capability subcommands.
#[derive(Subcommand)]
pub enum CapabilityCommand {
    /// List local capability registry rows.
    List,
    /// Verify a detached Ed25519 capability package signature.
    VerifySignature {
        /// Capability name.
        #[arg(long)]
        name: String,
        /// Capability version.
        #[arg(long)]
        version: String,
        /// Package checksum in sha256:<hex> form.
        #[arg(long)]
        checksum: String,
        /// Path to manifest JSON.
        #[arg(long)]
        manifest: PathBuf,
        /// Detached Ed25519 signature as hex.
        #[arg(long)]
        signature: String,
        /// Trusted key as key_id:hex_public_key. Repeat to pin more roots.
        #[arg(long = "trusted-key")]
        trusted_keys: Vec<String>,
    },
}

/// Run a capability registry subcommand.
pub async fn run(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    sub: CapabilityCommand,
) -> Result<()> {
    match sub {
        CapabilityCommand::List => list(client, bundle, output).await,
        CapabilityCommand::VerifySignature {
            name,
            version,
            checksum,
            manifest,
            signature,
            trusted_keys,
        } => {
            verify_signature(
                client,
                bundle,
                output,
                VerifyArgs {
                    name,
                    version,
                    checksum,
                    manifest,
                    signature,
                    trusted_keys,
                },
            )
            .await
        }
    }
}

async fn list(client: &Client, bundle: &Bundle, output: OutputMode) -> Result<()> {
    let body: Value = client.get("/v1/capabilities").await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&body)?),
        OutputMode::Plain => {
            let caps: Vec<Capability> = serde_json::from_value(body)?;
            println!("capabilities={}", caps.len());
        }
        OutputMode::Human => {
            let caps: Vec<Capability> = serde_json::from_value(body)?;
            render_human(bundle, &caps);
        }
    }
    Ok(())
}

async fn verify_signature(
    client: &Client,
    bundle: &Bundle,
    output: OutputMode,
    args: VerifyArgs,
) -> Result<()> {
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(args.manifest)?)?;
    let body = VerifyRequest {
        name: args.name,
        version: args.version,
        checksum: args.checksum,
        manifest,
        signature: args.signature,
        trusted_keys: parse_trusted_keys(args.trusted_keys)?,
    };
    let verified: SignatureVerification = client
        .post("/v1/capabilities/verify-signature", &body)
        .await?;
    match output {
        OutputMode::Json => println!("{}", serde_json::to_string_pretty(&verified)?),
        OutputMode::Plain => println!(
            "verified={}@{} key={}",
            verified.name, verified.version, verified.key_id
        ),
        OutputMode::Human => println!(
            "{}",
            bundle.t(
                "capability-signature-ok",
                &[
                    ("name", &verified.name),
                    ("version", &verified.version),
                    ("key", &verified.key_id),
                ],
            )
        ),
    }
    Ok(())
}

fn parse_trusted_keys(values: Vec<String>) -> Result<Vec<TrustedKey>> {
    values
        .into_iter()
        .map(|value| {
            let (key_id, public_key) = value
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("trusted key must be key_id:hex_public_key"))?;
            Ok(TrustedKey {
                key_id: key_id.into(),
                public_key: public_key.into(),
            })
        })
        .collect()
}

fn render_human(bundle: &Bundle, caps: &[Capability]) {
    if caps.is_empty() {
        println!("{}", bundle.t("capabilities-empty", &[]));
        return;
    }
    println!("{}", bundle.t("capabilities-header", &[]));
    for cap in caps {
        println!(
            "{}",
            bundle.t(
                "capability-line",
                &[
                    ("name", &cap.name),
                    ("version", &cap.version),
                    ("status", &cap.status),
                ],
            )
        );
    }
}

#[derive(Debug, Deserialize)]
struct Capability {
    name: String,
    version: String,
    status: String,
}

struct VerifyArgs {
    name: String,
    version: String,
    checksum: String,
    manifest: PathBuf,
    signature: String,
    trusted_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
struct VerifyRequest {
    name: String,
    version: String,
    checksum: String,
    manifest: Value,
    signature: String,
    trusted_keys: Vec<TrustedKey>,
}

#[derive(Debug, Serialize)]
struct TrustedKey {
    key_id: String,
    public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SignatureVerification {
    name: String,
    version: String,
    key_id: String,
}
