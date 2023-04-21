use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

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
    pub max_consecutive_timeouts: usize,
    pub stats: Arc<Mutex<Stats>>,
}

#[derive(Clone)]
pub struct Stats {
    /// Counts the number of times that the condition evaluates to true
    pub n_has_leader: usize,
    /// Counts the number of times that the condition evaluates to true
    pub n_max_term: usize,
    /// Counts the number of times that the condition evaluates to true
    pub n_max_consecutive_timeouts: usize,
}

impl Display for Stats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "has leader: {},\nmax term: {},\nmax consecutive timeouts: {}",
            self.n_has_leader, self.n_max_term, self.n_max_consecutive_timeouts
        )
    }
}

impl Default for Stats {
    fn default() -> Self {
        Stats { n_has_leader: 0, n_max_term: 0, n_max_consecutive_timeouts: 0 }
    }
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
            .within_boundary(|cfg, ams| {
                // Stateright can make actors timeout an unbounded amount of times.
                // For example, it can indefinitely prevent servers from electing a leader by repeatedly timing out candidates.
                // Unfortunately, by doing so, stateright also explores endless and uninteresting paths where the servers hardly make progress.
                // Therefore, we put a bound on the term that servers can reach.

                // Also see Ongaro's PhD thesis, Chapter 9:
                // According to the FLP impossibility result [28], no fault-tolerant consensus protocol can deterministically terminate in a purely asynchronous model.

                // do not check states where a server's term has exceeded the maximum one
                let below_max_term = ams.actor_states.iter().all(|state| state.current_term <= cfg.max_term);

                let below_max_timeouts = ams
                    .actor_states
                    .iter()
                    .map(|state| match &state.state {
                        State::Follower => 0,
                        State::Candidate(candidate) => candidate.n_consecutive_timeouts,
                        State::Leader(leader) => leader.n_consecutive_timeouts,
                    })
                    .all(|n_timeouts| n_timeouts <= cfg.max_consecutive_timeouts);

                below_max_term && below_max_timeouts
            })
            .property(Expectation::Always, "Election safety", |_, ams| {
                ams.actor_states
                    .iter()
                    .filter(|&state| matches!(state.state, State::Leader(_)))
                    .map(|state| state.current_term)
                    .all_unique()
            })
            .property(Expectation::Eventually, "A leader is elected", |am, ams| {
                // issues/flp.png shows an counterexample to this property where a server's term reaches the maximum (6) but no leader is elected.
                // Such counterexamples happen because stateright can indefinitely prevent servers from electing a leader by repeatedly timing out candidates.
                // To prevent stateright from reporting such counterexamples, we always return true when a server's term reaches the maximum but no leader is elected.
                let has_leader =
                    ams.actor_states.iter().find(|&state| matches!(state.state, State::Leader(_))).is_some();
                let max_term_reached = ams.actor_states.iter().any(|state| state.current_term == am.cfg.max_term);
                let max_consecutive_timeouts_reached = ams
                    .actor_states
                    .iter()
                    .map(|state| match &state.state {
                        State::Follower => 0,
                        State::Candidate(candidate) => candidate.n_consecutive_timeouts,
                        State::Leader(leader) => leader.n_consecutive_timeouts,
                    })
                    .any(|n_timeouts| n_timeouts == am.cfg.max_consecutive_timeouts);

                // update statistics
                if has_leader {
                    am.cfg.stats.lock().unwrap().n_has_leader += 1;
                } else if max_term_reached {
                    am.cfg.stats.lock().unwrap().n_max_term += 1;
                } else if max_consecutive_timeouts_reached {
                    am.cfg.stats.lock().unwrap().n_max_consecutive_timeouts += 1;
                }

                has_leader || max_term_reached || max_consecutive_timeouts_reached
            })
    }
}
