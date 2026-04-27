//! `AuditLog` writer + verifier.

use super::canonical::canonical_json;
use super::hash::{compute_hash, GENESIS_HASH};
use super::model::{AuditEntry, EntityKind, VerifyReport};
use crate::error::Result;
use chrono::Utc;
use convergio_db::Pool;
use serde::Serialize;
use uuid::Uuid;

/// Audit log handle. Cheap to clone (clones the underlying pool).
#[derive(Clone)]
pub struct AuditLog {
    pool: Pool,
}

impl AuditLog {
    /// Wrap a pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Append one entry to the log. Reads the previous `hash`, computes
    /// the new one, writes the row.
    pub async fn append<P: Serialize>(
        &self,
        entity: EntityKind,
        entity_id: &str,
        transition: &str,
        payload: &P,
        agent_id: Option<&str>,
    ) -> Result<AuditEntry> {
        let prev = self.tail().await?;
        let prev_hash = prev
            .as_ref()
            .map(|e| e.hash.as_str())
            .unwrap_or(GENESIS_HASH);
        let next_seq = prev.as_ref().map(|e| e.seq + 1).unwrap_or(1);

        let payload_str = canonical_json(payload)?;
        let hash = compute_hash(prev_hash, &payload_str);

        let entry = AuditEntry {
            id: Uuid::new_v4().to_string(),
            seq: next_seq,
            entity_type: entity.as_str().to_string(),
            entity_id: entity_id.to_string(),
            transition: transition.to_string(),
            payload: payload_str,
            agent_id: agent_id.map(str::to_string),
            prev_hash: prev_hash.to_string(),
            hash,
            created_at: Utc::now().to_rfc3339(),
        };

        sqlx::query(
            "INSERT INTO audit_log (id, seq, entity_type, entity_id, transition, \
             payload, agent_id, prev_hash, hash, created_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&entry.id)
        .bind(entry.seq)
        .bind(&entry.entity_type)
        .bind(&entry.entity_id)
        .bind(&entry.transition)
        .bind(&entry.payload)
        .bind(&entry.agent_id)
        .bind(&entry.prev_hash)
        .bind(&entry.hash)
        .bind(&entry.created_at)
        .execute(self.pool.inner())
        .await?;

        Ok(entry)
    }

    /// Last entry by `seq`, or `None` if the log is empty.
    pub async fn tail(&self) -> Result<Option<AuditEntry>> {
        let row = sqlx::query_as::<_, AuditRow>(
            "SELECT id, seq, entity_type, entity_id, transition, payload, agent_id, \
             prev_hash, hash, created_at FROM audit_log ORDER BY seq DESC LIMIT 1",
        )
        .fetch_optional(self.pool.inner())
        .await?;
        Ok(row.map(Into::into))
    }

    /// Verify the chain in `[from, to]` (both inclusive, both optional).
    pub async fn verify(&self, from: Option<i64>, to: Option<i64>) -> Result<VerifyReport> {
        let from_seq = from.unwrap_or(1);
        let to_seq = to.unwrap_or(i64::MAX);

        let mut prev_hash = self.bootstrap_prev_hash(from_seq).await?;
        let rows = sqlx::query_as::<_, (i64, String, String, String)>(
            "SELECT seq, payload, prev_hash, hash FROM audit_log \
             WHERE seq >= ? AND seq <= ? ORDER BY seq ASC",
        )
        .bind(from_seq)
        .bind(to_seq)
        .fetch_all(self.pool.inner())
        .await?;

        let mut checked = 0i64;
        for (seq, payload, row_prev, row_hash) in rows {
            checked += 1;
            if row_prev != prev_hash {
                return Ok(VerifyReport {
                    ok: false,
                    checked,
                    broken_at: Some(seq),
                });
            }
            if compute_hash(&prev_hash, &payload) != row_hash {
                return Ok(VerifyReport {
                    ok: false,
                    checked,
                    broken_at: Some(seq),
                });
            }
            prev_hash = row_hash;
        }
        Ok(VerifyReport {
            ok: true,
            checked,
            broken_at: None,
        })
    }

    async fn bootstrap_prev_hash(&self, from_seq: i64) -> Result<String> {
        if from_seq <= 1 {
            return Ok(GENESIS_HASH.to_string());
        }
        let row =
            sqlx::query_as::<_, (String,)>("SELECT hash FROM audit_log WHERE seq = ? LIMIT 1")
                .bind(from_seq - 1)
                .fetch_optional(self.pool.inner())
                .await?;
        Ok(row.map(|r| r.0).unwrap_or_else(|| GENESIS_HASH.to_string()))
    }
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: String,
    seq: i64,
    entity_type: String,
    entity_id: String,
    transition: String,
    payload: String,
    agent_id: Option<String>,
    prev_hash: String,
    hash: String,
    created_at: String,
}

impl From<AuditRow> for AuditEntry {
    fn from(r: AuditRow) -> Self {
        Self {
            id: r.id,
            seq: r.seq,
            entity_type: r.entity_type,
            entity_id: r.entity_id,
            transition: r.transition,
            payload: r.payload,
            agent_id: r.agent_id,
            prev_hash: r.prev_hash,
            hash: r.hash,
            created_at: r.created_at,
        }
    }
}
