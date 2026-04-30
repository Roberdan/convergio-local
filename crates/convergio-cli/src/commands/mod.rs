//! CLI subcommand modules — one file per top-level command.

pub mod audit;
pub mod capability;
mod capability_types;
pub mod crdt;
pub mod demo;
pub mod dispatch;
pub mod doctor;
pub mod evidence;
pub mod health;
pub mod mcp;
pub mod plan;
pub mod service;
pub mod setup;
pub mod solve;
pub mod status;
pub mod task;
pub mod validate;
pub mod workspace;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Global output rendering mode for commands that support multiple views.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputMode {
    /// Localized human output.
    Human,
    /// Pretty JSON for scripts and agents.
    Json,
    /// Minimal plain text for shell pipelines.
    Plain,
}

/// Tiny HTTP helper shared by subcommands.
pub struct Client {
    base: String,
    inner: reqwest::Client,
}

impl Client {
    /// Build with the daemon base URL (e.g. `http://127.0.0.1:8420`).
    pub fn new(base: String) -> Self {
        Self {
            base,
            inner: reqwest::Client::new(),
        }
    }

    /// Daemon base URL — used by localized error messages.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// `GET path` and parse the JSON body into `T`.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .inner
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        json_or_err(resp).await
    }

    /// `POST path` with `body` and parse the JSON body into `T`.
    pub async fn post<B: Serialize, T: DeserializeOwned>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .inner
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        json_or_err(resp).await
    }

    /// `DELETE path` and parse the JSON body into `T`.
    pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .inner
            .delete(&url)
            .send()
            .await
            .with_context(|| format!("DELETE {url}"))?;
        json_or_err(resp).await
    }
}

async fn json_or_err<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    let text = resp.text().await.context("reading response body")?;
    if !status.is_success() {
        anyhow::bail!("HTTP {status}: {text}");
    }
    serde_json::from_str(&text).with_context(|| format!("parsing JSON: {text}"))
}
