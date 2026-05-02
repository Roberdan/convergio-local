//! Cross-pane scope filtering.
//!
//! Whichever plan the cursor sits on in the Plans pane is the
//! *scope* the Tasks / Agents / PRs panes filter their content
//! against. This is the lazygit pattern: master selection drives
//! detail. The implementations live here so [`crate::state`] stays
//! under the 300-line cap.

use crate::client::{RegistryAgent, TaskSummary};
use crate::state::AppState;

impl AppState {
    /// Plan id currently scoped by the Plans-pane cursor, if any.
    pub fn scoped_plan_id(&self) -> Option<&str> {
        self.plans
            .get(self.cursor.plans.selected)
            .map(|p| p.id.as_str())
    }

    /// Plan title for the scoped plan. Used in pane title breadcrumbs.
    pub fn scoped_plan_title(&self) -> Option<&str> {
        self.plans
            .get(self.cursor.plans.selected)
            .map(|p| p.title.as_str())
    }

    /// Tasks of the scoped plan, derived from the active-tasks pool.
    /// Returns the full active-tasks vector when no plan is scoped.
    pub fn scoped_tasks(&self) -> Vec<&TaskSummary> {
        match self.scoped_plan_id() {
            Some(pid) => self.tasks.iter().filter(|t| t.plan_id == pid).collect(),
            None => self.tasks.iter().collect(),
        }
    }

    /// Agents that own at least one task in the scoped plan. With
    /// no scope, returns every registered agent.
    pub fn scoped_agents(&self) -> Vec<&RegistryAgent> {
        let Some(pid) = self.scoped_plan_id() else {
            return self.agents.iter().collect();
        };
        let owners: std::collections::HashSet<&str> = self
            .tasks
            .iter()
            .filter(|t| t.plan_id == pid)
            .filter_map(|t| t.agent_id.as_deref())
            .collect();
        if owners.is_empty() {
            return Vec::new();
        }
        self.agents
            .iter()
            .filter(|a| owners.contains(a.id.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Plan, RegistryAgent, TaskSummary};

    fn plan(id: &str, title: &str) -> Plan {
        Plan {
            id: id.into(),
            title: title.into(),
            project: None,
            status: "active".into(),
            updated_at: "2026-05-02".into(),
        }
    }

    fn task(id: &str, plan_id: &str, owner: Option<&str>) -> TaskSummary {
        TaskSummary {
            id: id.into(),
            plan_id: plan_id.into(),
            title: id.into(),
            status: "in_progress".into(),
            agent_id: owner.map(|s| s.into()),
        }
    }

    fn agent(id: &str) -> RegistryAgent {
        RegistryAgent {
            id: id.into(),
            kind: "claude".into(),
            status: Some("idle".into()),
            last_heartbeat_at: None,
        }
    }

    #[test]
    fn scoped_tasks_filters_to_selected_plan() {
        let mut s = AppState {
            plans: vec![plan("p1", "P1"), plan("p2", "P2")],
            tasks: vec![
                task("t1", "p1", None),
                task("t2", "p2", None),
                task("t3", "p1", None),
            ],
            ..AppState::default()
        };
        s.cursor.plans.selected = 0;
        let scoped: Vec<&str> = s.scoped_tasks().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(scoped, vec!["t1", "t3"]);
        s.cursor.plans.selected = 1;
        let scoped: Vec<&str> = s.scoped_tasks().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(scoped, vec!["t2"]);
    }

    #[test]
    fn scoped_agents_filters_to_owners_of_scoped_tasks() {
        let mut s = AppState {
            plans: vec![plan("p1", "P1"), plan("p2", "P2")],
            tasks: vec![
                task("t1", "p1", Some("alpha")),
                task("t2", "p2", Some("beta")),
            ],
            agents: vec![agent("alpha"), agent("beta"), agent("gamma")],
            ..AppState::default()
        };
        s.cursor.plans.selected = 0;
        let scoped: Vec<&str> = s.scoped_agents().iter().map(|a| a.id.as_str()).collect();
        assert_eq!(scoped, vec!["alpha"]);
        s.cursor.plans.selected = 1;
        let scoped: Vec<&str> = s.scoped_agents().iter().map(|a| a.id.as_str()).collect();
        assert_eq!(scoped, vec!["beta"]);
    }
}
