//! MCP tool declarations for the bridge.

use crate::help;
use crate::http::fallback_error;
use convergio_api::{ActRequest, Action, HelpRequest, HelpTopic, HelpVerbosity};
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct Bridge {
    pub(crate) url: String,
    pub(crate) client: reqwest::Client,
    pub(crate) last_refusal: Arc<Mutex<Option<Value>>>,
    pub(crate) tool_router: ToolRouter<Self>,
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct HelpParams {
    /// Help topic.
    #[serde(default = "default_help_topic")]
    topic: HelpTopic,
    /// Action details to return when topic is `action`.
    #[serde(default)]
    action: Option<Action>,
    /// Verbosity level.
    #[serde(default = "default_help_verbosity")]
    verbosity: HelpVerbosity,
}

impl From<HelpParams> for HelpRequest {
    fn from(value: HelpParams) -> Self {
        Self {
            topic: value.topic,
            action: value.action,
            verbosity: value.verbosity,
        }
    }
}

#[derive(Debug, serde::Deserialize, JsonSchema)]
struct ActParams {
    /// Schema version returned by `convergio.help`.
    schema_version: String,
    /// Action to execute.
    action: Action,
    /// Action-specific input.
    #[serde(default)]
    params: Value,
}

impl From<ActParams> for ActRequest {
    fn from(value: ActParams) -> Self {
        Self {
            schema_version: value.schema_version,
            action: value.action,
            params: value.params,
        }
    }
}

impl Bridge {
    pub(crate) fn new(url: String) -> Self {
        Self {
            url,
            client: reqwest::Client::new(),
            last_refusal: Arc::new(Mutex::new(None)),
            tool_router: Self::tool_router(),
        }
    }

    pub(crate) fn log_action(&self, action: Action, response: &convergio_api::AgentResponse) {
        let Ok(path) = mcp_log_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        trim_log(&path);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default();
        let line = serde_json::json!({
            "ts": ts,
            "action": action.as_str(),
            "ok": response.ok,
            "code": response.code,
            "next": response.next,
        });
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

fn mcp_log_path() -> Result<PathBuf, std::env::VarError> {
    Ok(PathBuf::from(std::env::var("HOME")?).join(".convergio/mcp.log"))
}

fn trim_log(path: &std::path::Path) {
    const MAX_BYTES: u64 = 256 * 1024;
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    if meta.len() <= MAX_BYTES {
        return;
    }
    if let Ok(content) = fs::read_to_string(path) {
        let keep: Vec<&str> = content.lines().rev().take(1000).collect();
        let trimmed = keep.into_iter().rev().collect::<Vec<_>>().join("\n");
        let _ = fs::write(path, format!("{trimmed}\n"));
    }
}

#[tool_router(router = tool_router)]
impl Bridge {
    #[tool(
        name = "convergio.help",
        description = "Read Convergio agent protocol help."
    )]
    async fn help(&self, Parameters(params): Parameters<HelpParams>) -> String {
        serde_json::to_string(&help::response(&HelpRequest::from(params)))
            .unwrap_or_else(|e| fallback_error(format!("failed to serialize help response: {e}")))
    }

    #[tool(
        name = "convergio.act",
        description = "Execute one typed Convergio action."
    )]
    async fn act(&self, Parameters(params): Parameters<ActParams>) -> String {
        let response = self.dispatch(ActRequest::from(params)).await;
        serde_json::to_string(&response)
            .unwrap_or_else(|e| fallback_error(format!("failed to serialize action response: {e}")))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Bridge {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Use convergio.help once, then convergio.act with typed actions.")
    }
}

fn default_help_topic() -> HelpTopic {
    HelpTopic::Quickstart
}

fn default_help_verbosity() -> HelpVerbosity {
    HelpVerbosity::Short
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_exact_two_tools() {
        let bridge = Bridge::new("http://127.0.0.1:8420".into());
        let tools = bridge.tool_router.list_all();
        let names: Vec<String> = tools.into_iter().map(|t| t.name.to_string()).collect();
        assert_eq!(names, vec!["convergio.act", "convergio.help"]);
    }
}
