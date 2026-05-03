-- 0010_plan_pr_links.sql
--
-- ADR-0031: plan ↔ PR linkage table.
--
-- Used by the TUI dashboard's drill-down (`cvg dash` Plans → PRs)
-- and the planned `cvg session pre-stop check.plan_pr_drift`. Today
-- the only signal we have is "Tracks: T<task_id>" parsed from PR
-- titles or branch names containing the plan id; both are noisy.
-- This table is the canonical source: when present, it overrides
-- the heuristics.

CREATE TABLE IF NOT EXISTS plan_pr_links (
    id          TEXT PRIMARY KEY,            -- UUIDv4
    plan_id     TEXT NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    task_id     TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    pr_number   INTEGER NOT NULL,
    repo_slug   TEXT NOT NULL,               -- e.g. "Roberdan/convergio"
    branch      TEXT,                         -- best-effort, may be NULL when only the # is known
    created_at  TEXT NOT NULL,
    UNIQUE(repo_slug, pr_number, plan_id)
);

CREATE INDEX IF NOT EXISTS idx_plan_pr_links_plan ON plan_pr_links(plan_id);
CREATE INDEX IF NOT EXISTS idx_plan_pr_links_task ON plan_pr_links(task_id);
CREATE INDEX IF NOT EXISTS idx_plan_pr_links_pr ON plan_pr_links(repo_slug, pr_number);
