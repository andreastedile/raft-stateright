use std::borrow::Cow;

use stateright::actor::Out;

use crate::server::RaftServer;
use crate::state::{RaftState, State};
use crate::timers::RaftTimer;
use crate::types::Term;

impl RaftServer {
    pub fn update_term(&self, state: &mut Cow<RaftState>, new_term: Term, o: &mut Out<Self>) {
        if new_term > state.current_term {
            let state = state.to_mut();

            state.current_term = new_term;

            match state.state {
                State::Follower => {
                    o.cancel_timer(RaftTimer::Election);
                }
                State::Candidate(_) => {
                    o.cancel_timer(RaftTimer::Election);
                    state.state = State::Follower;
                }
                State::Leader(_) => {
                    o.cancel_timer(RaftTimer::Heartbeat);
                    state.state = State::Follower;
                }
            }

            state.voted_for = None;

            o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
        }
    }
}
