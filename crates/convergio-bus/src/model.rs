//! Domain types for Layer 2.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A persisted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// UUID v4.
    pub id: String,
    /// Monotonic global sequence (1-based).
    pub seq: i64,
    /// Plan that owns this message.
    pub plan_id: String,
    /// Free-form topic (e.g. `task.done`, `agent:foo` for direct).
    pub topic: String,
    /// Agent id of the publisher, or `None` for system messages.
    pub sender: Option<String>,
    /// Caller-supplied JSON payload.
    pub payload: serde_json::Value,
    /// When the consumer acked this message; `None` if still in flight.
    pub consumed_at: Option<DateTime<Utc>>,
    /// Agent id of the consumer that acked, if any.
    pub consumed_by: Option<String>,
    /// Publish timestamp.
    pub created_at: DateTime<Utc>,
}

/// Input for [`crate::Bus::publish`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMessage {
    /// Plan that owns this message.
    pub plan_id: String,
    /// Topic.
    pub topic: String,
    /// Sender (publisher) agent id, if any.
    #[serde(default)]
    pub sender: Option<String>,
    /// Payload.
    pub payload: serde_json::Value,
}
