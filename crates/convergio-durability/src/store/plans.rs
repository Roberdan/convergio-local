//! `plans` table DAO.

use crate::error::{DurabilityError, Result};
use crate::model::{NewPlan, Plan, PlanStatus};
use chrono::{DateTime, Utc};
use convergio_db::Pool;
use uuid::Uuid;

/// Read/write access to the `plans` table.
#[derive(Clone)]
pub struct PlanStore {
    pool: Pool,
}

impl PlanStore {
    /// Wrap a pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Insert a new plan with status `draft`.
    pub async fn create(&self, input: NewPlan) -> Result<Plan> {
        let now = Utc::now();
        let plan = Plan {
            id: Uuid::new_v4().to_string(),
            title: input.title,
            description: input.description,
            project: input.project,
            status: PlanStatus::Draft,
            created_at: now,
            updated_at: now,
            started_at: None,
            ended_at: None,
            duration_ms: None,
        };

        sqlx::query(
            "INSERT INTO plans (id, title, description, project, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&plan.id)
        .bind(&plan.title)
        .bind(&plan.description)
        .bind(&plan.project)
        .bind(plan.status.as_str())
        .bind(plan.created_at.to_rfc3339())
        .bind(plan.updated_at.to_rfc3339())
        .execute(self.pool.inner())
        .await?;

        Ok(plan)
    }

    /// Fetch by id, or `NotFound`.
    pub async fn get(&self, id: &str) -> Result<Plan> {
        self.find(id)
            .await?
            .ok_or_else(|| DurabilityError::NotFound {
                entity: "plan",
                id: id.to_string(),
            })
    }

    /// Fetch by id, returning `None` if absent.
    pub async fn find(&self, id: &str) -> Result<Option<Plan>> {
        let row = sqlx::query_as::<_, PlanRow>(
            "SELECT id, title, description, project, status, created_at, updated_at, \
             started_at, ended_at, duration_ms \
             FROM plans WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(self.pool.inner())
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    /// List plans, newest first.
    pub async fn list(&self, limit: i64) -> Result<Vec<Plan>> {
        let rows = sqlx::query_as::<_, PlanRow>(
            "SELECT id, title, description, project, status, created_at, updated_at, \
             started_at, ended_at, duration_ms \
             FROM plans ORDER BY created_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(self.pool.inner())
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    /// Update the status column. Caller is responsible for running the
    /// gate pipeline before calling.
    pub async fn set_status(&self, id: &str, status: PlanStatus) -> Result<()> {
        let n = sqlx::query("UPDATE plans SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(Utc::now().to_rfc3339())
            .bind(id)
            .execute(self.pool.inner())
            .await?
            .rows_affected();
        if n == 0 {
            return Err(DurabilityError::NotFound {
                entity: "plan",
                id: id.to_string(),
            });
        }
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: String,
    title: String,
    description: Option<String>,
    project: Option<String>,
    status: String,
    created_at: String,
    updated_at: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    duration_ms: Option<i64>,
}

impl TryFrom<PlanRow> for Plan {
    type Error = DurabilityError;
    fn try_from(r: PlanRow) -> Result<Self> {
        Ok(Plan {
            id: r.id,
            title: r.title,
            description: r.description,
            project: r.project,
            status: PlanStatus::parse(&r.status).unwrap_or(PlanStatus::Draft),
            created_at: parse_ts(&r.created_at)?,
            updated_at: parse_ts(&r.updated_at)?,
            started_at: r.started_at.as_deref().map(parse_ts).transpose()?,
            ended_at: r.ended_at.as_deref().map(parse_ts).transpose()?,
            duration_ms: r.duration_ms,
        })
    }
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| DurabilityError::NotFound {
            entity: "timestamp",
            id: s.to_string(),
        })
}
