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
    pub max_crashes: usize,
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
            .max_crashes(self.max_crashes)
            .within_boundary(|cfg, ams| {
                // Stateright can make actors timeout an unbounded amount of times.
                // For example, it can indefinitely prevent servers from electing a leader by repeatedly timing out candidates.
                // Unfortunately, by doing so, stateright also explores endless and uninteresting paths where the servers hardly make progress.
                // Therefore, we put a bound on the term that servers can reach.

                // Also see Ongaro's PhD thesis, Chapter 9:
                // According to the FLP impossibility result [28], no fault-tolerant consensus protocol can deterministically terminate in a purely asynchronous model.

                // do not check states where a server's term has exceeded the maximum one
                ams.actor_states.iter().all(|state| state.current_term <= cfg.max_term)
            })
            .property(Expectation::Always, "Election safety", |_, ams| {
                ams.actor_states
                    .iter()
                    .filter(|&state| matches!(state.state, State::Leader))
                    .map(|state| state.current_term)
                    .all_unique()
            })
            .property(Expectation::Eventually, "A leader is elected", |am, ams| {
                // issues/flp.png shows an counterexample to this property where a server's term reaches the maximum (6) but no leader is elected.
                // Such counterexamples happen because stateright can indefinitely prevent servers from electing a leader by repeatedly timing out candidates.
                // To prevent stateright from reporting such counterexamples, we always return true when a server's term reaches the maximum but no leader is elected.
                let has_leader = ams.actor_states.iter().find(|&state| matches!(state.state, State::Leader)).is_some();
                let has_reached_max_term = ams.actor_states.iter().any(|state| state.current_term == am.cfg.max_term);
                has_leader || has_reached_max_term
            })
    }
}
