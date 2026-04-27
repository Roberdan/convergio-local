//! Bus — write/read API for Layer 2.

use crate::error::{BusError, Result};
use crate::model::{Message, NewMessage};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use uuid::Uuid;

/// Read/write access to the message bus.
#[derive(Clone)]
pub struct Bus {
    pool: Pool,
}

impl Bus {
    /// Wrap a pool. The caller is responsible for having run
    /// [`crate::init`] at least once.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Append a message to the bus.
    pub async fn publish(&self, msg: NewMessage) -> Result<Message> {
        let payload = serde_json::to_string(&msg.payload)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let next_seq = next_seq(&self.pool).await?;

        sqlx::query(
            "INSERT INTO agent_messages \
             (id, seq, plan_id, topic, sender, payload, consumed_at, consumed_by, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, NULL, NULL, ?)",
        )
        .bind(&id)
        .bind(next_seq)
        .bind(&msg.plan_id)
        .bind(&msg.topic)
        .bind(&msg.sender)
        .bind(&payload)
        .bind(&now_str)
        .execute(self.pool.inner())
        .await?;

        Ok(Message {
            id,
            seq: next_seq,
            plan_id: msg.plan_id,
            topic: msg.topic,
            sender: msg.sender,
            payload: msg.payload,
            consumed_at: None,
            consumed_by: None,
            created_at: now,
        })
    }

    /// Poll unconsumed messages for `(plan_id, topic)` since `cursor`
    /// (exclusive). Returns up to `limit` rows in `seq` order.
    ///
    /// Pass `cursor = 0` on first call. The next cursor is the highest
    /// `seq` you saw. The bus does **not** auto-ack — call [`Self::ack`]
    /// when you have processed a message.
    pub async fn poll(
        &self,
        plan_id: &str,
        topic: &str,
        cursor: i64,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = sqlx::query_as::<_, MessageRow>(
            "SELECT id, seq, plan_id, topic, sender, payload, consumed_at, \
                    consumed_by, created_at \
             FROM agent_messages \
             WHERE plan_id = ? AND topic = ? AND seq > ? AND consumed_at IS NULL \
             ORDER BY seq ASC LIMIT ?",
        )
        .bind(plan_id)
        .bind(topic)
        .bind(cursor)
        .bind(limit)
        .fetch_all(self.pool.inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// Acknowledge consumption of a message. Idempotent — re-acking a
    /// message that's already acked is a no-op.
    pub async fn ack(&self, message_id: &str, consumer: Option<&str>) -> Result<()> {
        let n = sqlx::query(
            "UPDATE agent_messages SET consumed_at = ?, consumed_by = ? \
             WHERE id = ? AND consumed_at IS NULL",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(consumer)
        .bind(message_id)
        .execute(self.pool.inner())
        .await?
        .rows_affected();
        if n == 0 {
            // Either already acked, or nonexistent. Differentiate.
            let exists: Option<(String,)> =
                sqlx::query_as("SELECT id FROM agent_messages WHERE id = ? LIMIT 1")
                    .bind(message_id)
                    .fetch_optional(self.pool.inner())
                    .await?;
            if exists.is_none() {
                return Err(BusError::NotFound(message_id.to_string()));
            }
        }
        Ok(())
    }
}

async fn next_seq(pool: &Pool) -> Result<i64> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT MAX(seq) FROM agent_messages")
        .fetch_optional(pool.inner())
        .await?;
    Ok(row.map(|r| r.0).unwrap_or(0) + 1)
}

#[derive(sqlx::FromRow)]
struct MessageRow {
    id: String,
    seq: i64,
    plan_id: String,
    topic: String,
    sender: Option<String>,
    payload: String,
    consumed_at: Option<String>,
    consumed_by: Option<String>,
    created_at: String,
}

impl TryFrom<MessageRow> for Message {
    type Error = BusError;
    fn try_from(r: MessageRow) -> Result<Self> {
        Ok(Message {
            id: r.id,
            seq: r.seq,
            plan_id: r.plan_id,
            topic: r.topic,
            sender: r.sender,
            payload: serde_json::from_str(&r.payload)?,
            consumed_at: r.consumed_at.as_deref().and_then(parse_ts_opt),
            consumed_by: r.consumed_by,
            created_at: parse_ts(&r.created_at)?,
        })
    }
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| BusError::NotFound(format!("bad timestamp: {s}")))
}

fn parse_ts_opt(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}
