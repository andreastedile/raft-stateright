use std::borrow::Cow;

use stateright::actor::{Id, Out};

use crate::messages::{append_entries, RaftMsg};
use crate::server::RaftServer;
use crate::state::RaftState;
use crate::timers::RaftTimer;

impl RaftServer {
    pub fn on_heartbeat_timeout(&self, state: &mut Cow<RaftState>, o: &mut Out<Self>) {
        match state.as_ref() {
            RaftState::Follower { .. } => unreachable!(),
            RaftState::Candidate { .. } => unreachable!(),
            RaftState::Leader { current_term, .. } => {
                let req = append_entries::Request { term: *current_term };
                let req = RaftMsg::AppendEntriesReq(req);
                o.broadcast(&self.peers, &req);

                o.set_timer(RaftTimer::Heartbeat, self.config.heartbeat_period..self.config.heartbeat_period);
            }
        }
    }

    pub fn handle_append_entries_request(
        &self,
        state: &mut Cow<RaftState>,
        src: Id,
        req: append_entries::Request,
        o: &mut Out<Self>,
    ) {
        self.update_term(state, req.term, o);

        let res = append_entries::Response { term: state.current_term(), success: req.term >= state.current_term() };

        if req.term >= state.current_term() {
            o.cancel_timer(RaftTimer::Election);
            o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
        }

        o.send(src, RaftMsg::AppendEntriesRes(res));
    }

    pub fn handle_append_entries_response(
        &self,
        state: &mut Cow<RaftState>,
        _src: Id,
        res: append_entries::Response,
        o: &mut Out<Self>,
    ) {
        self.update_term(state, res.term, o);

        if res.term < state.current_term() {
            // stale response
        } else if let RaftState::Leader { .. } = state.as_ref() {
            //
        }
    }
}
