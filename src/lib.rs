pub mod config;
pub mod election;
pub mod messages;
pub mod replication;
pub mod server;
pub mod state;
pub mod timers;
pub mod types;
pub mod update_term;

mod tests {
    use itertools::Itertools;
    use stateright::actor::{model_peers, Actor, ActorModel, LossyNetwork, Network};
    use stateright::{Expectation, Model};

    use crate::server::RaftServer;
    use crate::state::RaftState;

    #[derive(Clone)]
    pub struct RaftModelCfg {
        server_count: usize,
        network: Network<<RaftServer as Actor>::Msg>,
        lossy_network: LossyNetwork,
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
                        .filter(|&state| matches!(state.as_ref(), RaftState::Leader { .. }))
                        .map(|state| state.current_term())
                        .all_unique()
                })
                .property(Expectation::Eventually, "A leader is elected", |_, ams| {
                    ams.actor_states.iter().find(|&state| matches!(state.as_ref(), RaftState::Leader { .. })).is_some()
                })
        }
    }

    #[test]
    fn explore() {
        RaftModelCfg { server_count: 2, network: Network::new_ordered([]), lossy_network: LossyNetwork::No }
            .into_model()
            .checker()
            .threads(8)
            .serve("localhost:3000");
    }
}
