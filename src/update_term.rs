use std::borrow::Cow;

use stateright::actor::Out;

use crate::server::RaftServer;
use crate::state::RaftState;
use crate::timers::RaftTimer;
use crate::types::Term;

impl RaftServer {
    pub fn update_term(&self, state: &mut Cow<RaftState>, new_term: Term, o: &mut Out<Self>) {
        if new_term > state.current_term() {
            match state.as_ref() {
                RaftState::Follower { .. } => {
                    *state.to_mut() = RaftState::Follower { current_term: new_term, voted_for: None };
                    o.cancel_timer(RaftTimer::Election);
                }
                RaftState::Candidate { .. } => {
                    o.cancel_timer(RaftTimer::Election);
                    *state.to_mut() = RaftState::Follower { current_term: new_term, voted_for: None };
                }
                RaftState::Leader { .. } => {
                    o.cancel_timer(RaftTimer::Heartbeat);
                    *state.to_mut() = RaftState::Follower { current_term: new_term, voted_for: None };
                }
            }

            o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
        }
    }
}
