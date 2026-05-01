//! Bus — write/read API for Layer 2.

use crate::error::{BusError, Result};
use crate::model::{Message, NewMessage, TopicSummary};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use uuid::Uuid;

/// Topic prefix that marks the system-scoped family (ADR-0024).
pub(crate) const SYSTEM_TOPIC_PREFIX: &str = "system.";

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

    /// Pool accessor for sibling impl blocks (e.g. [`crate::bus_system`]).
    pub(crate) fn pool(&self) -> &Pool {
        &self.pool
    }

    /// Append a plan-scoped message to the bus. The topic MUST NOT
    /// start with `system.` — those go through [`Self::publish_system`].
    pub async fn publish(&self, msg: NewMessage) -> Result<Message> {
        if msg.topic.starts_with(SYSTEM_TOPIC_PREFIX) {
            return Err(BusError::InvalidTopicScope(format!(
                "topic '{}' is system-scoped; use publish_system",
                msg.topic
            )));
        }
        let payload = serde_json::to_string(&msg.payload)?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_str = now.to_rfc3339();
        let mut tx = self.pool.inner().begin().await?;
        let next_seq = next_seq(&mut tx).await?;

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
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(Message {
            id,
            seq: next_seq,
            plan_id: Some(msg.plan_id),
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
    /// For system-scoped messages (`plan_id IS NULL`) use
    /// [`Self::poll_system`] instead.
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
        if topic.starts_with(SYSTEM_TOPIC_PREFIX) {
            return Err(BusError::InvalidTopicScope(format!(
                "topic '{topic}' is system-scoped; use poll_system"
            )));
        }
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

    /// Read every message for `plan_id` since `cursor` (exclusive),
    /// regardless of consumed status. Optional `topic` filter narrows
    /// the result. Designed for human-facing inspection (`cvg bus
    /// tail`); agents should keep using [`Self::poll`] which only
    /// returns unconsumed rows.
    pub async fn tail(
        &self,
        plan_id: &str,
        topic: Option<&str>,
        cursor: i64,
        limit: i64,
    ) -> Result<Vec<Message>> {
        let rows = if let Some(t) = topic {
            sqlx::query_as::<_, MessageRow>(
                "SELECT id, seq, plan_id, topic, sender, payload, consumed_at, \
                        consumed_by, created_at \
                 FROM agent_messages \
                 WHERE plan_id = ? AND topic = ? AND seq > ? \
                 ORDER BY seq ASC LIMIT ?",
            )
            .bind(plan_id)
            .bind(t)
            .bind(cursor)
            .bind(limit)
            .fetch_all(self.pool.inner())
            .await?
        } else {
            sqlx::query_as::<_, MessageRow>(
                "SELECT id, seq, plan_id, topic, sender, payload, consumed_at, \
                        consumed_by, created_at \
                 FROM agent_messages \
                 WHERE plan_id = ? AND seq > ? \
                 ORDER BY seq ASC LIMIT ?",
            )
            .bind(plan_id)
            .bind(cursor)
            .bind(limit)
            .fetch_all(self.pool.inner())
            .await?
        };
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// Per-topic summary for `plan_id`: total message count, the most
    /// recent `seq`, and the last `created_at`. Returned in
    /// alphabetic topic order so output is stable.
    pub async fn topics(&self, plan_id: &str) -> Result<Vec<TopicSummary>> {
        let rows: Vec<(String, i64, i64, String)> = sqlx::query_as(
            "SELECT topic, COUNT(*) AS count, MAX(seq) AS last_seq, MAX(created_at) AS last_at \
             FROM agent_messages \
             WHERE plan_id = ? \
             GROUP BY topic \
             ORDER BY topic ASC",
        )
        .bind(plan_id)
        .fetch_all(self.pool.inner())
        .await?;
        Ok(rows
            .into_iter()
            .map(|(topic, count, last_seq, last_at)| TopicSummary {
                topic,
                count,
                last_seq,
                last_at,
            })
            .collect())
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

pub(crate) async fn next_seq(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "UPDATE agent_message_sequence SET next_seq = next_seq + 1 \
         WHERE id = 1 RETURNING next_seq - 1",
    )
    .fetch_one(&mut **tx)
    .await?;
    Ok(row.0)
}

#[derive(sqlx::FromRow)]
pub(crate) struct MessageRow {
    id: String,
    seq: i64,
    plan_id: Option<String>,
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
