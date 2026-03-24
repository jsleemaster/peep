use crate::protocol::types::AgentState;
use crate::store::state::AppStore;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DerivedMetrics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub waiting_agents: usize,
    pub completed_agents: usize,
    pub total_events: usize,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_context_percent: f64,
    pub velocity_per_min: usize,
}

impl AppStore {
    pub fn derived_metrics(&self, now: i64) -> DerivedMetrics {
        let agents: Vec<_> = self.agents.values().collect();
        let total_agents = agents.len();
        let active_agents = agents.iter().filter(|a| a.state == AgentState::Active).count();
        let waiting_agents = agents.iter().filter(|a| a.state == AgentState::Waiting).count();
        let completed_agents = agents
            .iter()
            .filter(|a| a.state == AgentState::Completed)
            .count();
        let total_events = self.feed.len();
        let total_tokens: u64 = agents.iter().map(|a| a.total_tokens).sum();
        let total_cost: f64 = agents.iter().filter_map(|a| a.cost_usd).sum();

        let ctx_agents: Vec<f64> = agents.iter().filter_map(|a| a.context_percent).collect();
        let avg_context_percent = if ctx_agents.is_empty() {
            0.0
        } else {
            ctx_agents.iter().sum::<f64>() / ctx_agents.len() as f64
        };

        let velocity_per_min = self.velocity_per_min(now);

        DerivedMetrics {
            total_agents,
            active_agents,
            waiting_agents,
            completed_agents,
            total_events,
            total_tokens,
            total_cost,
            avg_context_percent,
            velocity_per_min,
        }
    }
}
