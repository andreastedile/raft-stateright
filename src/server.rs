use std::borrow::Cow;

use stateright::actor::{Actor, Id, Out};

use crate::config::RaftConfig;
use crate::messages::RaftMsg;
use crate::state::{RaftState, State};
use crate::timers::RaftTimer;

#[derive(Debug)]
pub struct RaftServer {
    pub id: Id,
    pub config: RaftConfig,
    pub peers: Vec<Id>,
}

impl Actor for RaftServer {
    type Msg = RaftMsg;
    type Timer = RaftTimer;
    type State = RaftState;

    fn on_start(&self, id: Id, o: &mut Out<Self>) -> Self::State {
        if self.peers.is_empty() {
            RaftState { current_term: 0, voted_for: Some(id), state: State::Leader }
        } else {
            o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());

            RaftState { current_term: 0, voted_for: None, state: State::Follower }
        }
    }

    fn on_msg(&self, _: Id, state: &mut Cow<Self::State>, src: Id, msg: Self::Msg, o: &mut Out<Self>) {
        match msg {
            RaftMsg::RequestVoteReq(req) => self.handle_request_vote_request(state, src, req, o),
            RaftMsg::RequestVoteRes(res) => self.handle_request_vote_response(state, src, res, o),
            RaftMsg::AppendEntriesReq(req) => self.handle_append_entries_request(state, src, req, o),
            RaftMsg::AppendEntriesRes(res) => self.handle_append_entries_response(state, src, res, o),
        }
    }

    fn on_timeout(&self, _: Id, state: &mut Cow<Self::State>, timeout: &Self::Timer, o: &mut Out<Self>) {
        match timeout {
            RaftTimer::Election => self.on_election_timeout(state, o),
            RaftTimer::Heartbeat => self.on_heartbeat_timeout(state, o),
        }
    }
}
