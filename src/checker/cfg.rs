use itertools::Itertools;
use stateright::actor::{model_peers, Actor, ActorModel, LossyNetwork, Network};
use stateright::Expectation;

use crate::server::RaftServer;
use crate::state::State;
use crate::types::Term;

#[derive(Clone)]
pub struct RaftModelCfg {
    pub server_count: usize,
    pub network: Network<<RaftServer as Actor>::Msg>,
    pub lossy_network: LossyNetwork,
    pub max_term: Term,
}

impl RaftModelCfg {
    pub fn into_model(self) -> ActorModel<RaftServer, Self, ()> {
        ActorModel::new(self.clone(), ())
            .actors((0..self.server_count).map(|i| RaftServer {
                id: i.into(),
                config: Default::default(),
                peers: model_peers(i, self.server_count),
            }))
            .init_network(self.network)
            .lossy_network(self.lossy_network)
            .property(Expectation::Always, "Election safety", |_, ams| {
                ams.actor_states
                    .iter()
                    .filter(|&state| matches!(state.state, State::Leader))
                    .map(|state| state.current_term)
                    .all_unique()
            })
            .property(Expectation::Eventually, "A leader is elected", |am, ams| {
                let has_leader = ams.actor_states.iter().find(|&state| matches!(state.state, State::Leader)).is_some();
                let reached_max_term = ams.actor_states.iter().any(|state| state.current_term == am.cfg.max_term);
                has_leader || reached_max_term
            })
            .within_boundary(|cfg, ams| ams.actor_states.iter().all(|state| state.current_term <= cfg.max_term))
    }
}
