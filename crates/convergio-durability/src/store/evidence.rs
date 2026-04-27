//! `evidence` table DAO.

use crate::error::{DurabilityError, Result};
use crate::model::Evidence;
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use uuid::Uuid;

/// Read/write access to the `evidence` table.
#[derive(Clone)]
pub struct EvidenceStore {
    pool: Pool,
}

impl EvidenceStore {
    /// Wrap a pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Attach evidence to a task.
    pub async fn attach(
        &self,
        task_id: &str,
        kind: &str,
        payload: serde_json::Value,
        exit_code: Option<i64>,
    ) -> Result<Evidence> {
        let evidence = Evidence {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            kind: kind.to_string(),
            payload,
            exit_code,
            created_at: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO evidence (id, task_id, kind, payload, exit_code, created_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&evidence.id)
        .bind(&evidence.task_id)
        .bind(&evidence.kind)
        .bind(serde_json::to_string(&evidence.payload)?)
        .bind(evidence.exit_code)
        .bind(evidence.created_at.to_rfc3339())
        .execute(self.pool.inner())
        .await?;

        Ok(evidence)
    }

    /// All evidence rows for a task, oldest first.
    pub async fn list_by_task(&self, task_id: &str) -> Result<Vec<Evidence>> {
        let rows = sqlx::query_as::<_, EvidenceRow>(
            "SELECT id, task_id, kind, payload, exit_code, created_at FROM evidence \
             WHERE task_id = ? ORDER BY created_at ASC",
        )
        .bind(task_id)
        .fetch_all(self.pool.inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// Names of evidence kinds present for a task (deduplicated).
    pub async fn kinds_for(&self, task_id: &str) -> Result<Vec<String>> {
        let rows =
            sqlx::query_as::<_, (String,)>("SELECT DISTINCT kind FROM evidence WHERE task_id = ?")
                .bind(task_id)
                .fetch_all(self.pool.inner())
                .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}

#[derive(sqlx::FromRow)]
struct EvidenceRow {
    id: String,
    task_id: String,
    kind: String,
    payload: String,
    exit_code: Option<i64>,
    created_at: String,
}

impl TryFrom<EvidenceRow> for Evidence {
    type Error = DurabilityError;
    fn try_from(r: EvidenceRow) -> Result<Self> {
        Ok(Evidence {
            id: r.id,
            task_id: r.task_id,
            kind: r.kind,
            payload: serde_json::from_str(&r.payload)?,
            exit_code: r.exit_code,
            created_at: DateTime::parse_from_rfc3339(&r.created_at)
                .map(|d| d.with_timezone(&Utc))
                .map_err(|_| DurabilityError::NotFound {
                    entity: "timestamp",
                    id: r.created_at.clone(),
                })?,
        })
    }
}
