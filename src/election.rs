use std::borrow::Cow;

use stateright::actor::{majority, Id, Out};

use crate::messages::{append_entries, request_vote, RaftMsg};
use crate::server::RaftServer;
use crate::state::{candidate_state, leader_state, RaftState, State};
use crate::timers::RaftTimer;

impl RaftServer {
    pub fn on_election_timeout(&self, state: &mut Cow<RaftState>, o: &mut Out<Self>) {
        let state = state.to_mut();

        match &mut state.state {
            State::Follower => {
                state.state = State::Candidate(candidate_state::State::default());
                state.current_term += 1;
                state.voted_for = Some(self.id);

                let req = request_vote::Request { term: state.current_term };
                let req = RaftMsg::RequestVoteReq(req);
                o.broadcast(&self.peers, &req);

                o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
            }
            State::Candidate(candidate) => {
                state.current_term += 1;
                candidate.votes.clear();
                candidate.n_consecutive_timeouts += 1;

                let req = request_vote::Request { term: state.current_term };
                let req = RaftMsg::RequestVoteReq(req);
                o.broadcast(&self.peers, &req);

                o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
            }
            State::Leader(_) => unreachable!(),
        }
    }

    pub fn handle_request_vote_request(
        &self,
        state: &mut Cow<RaftState>,
        src: Id,
        req: request_vote::Request,
        o: &mut Out<Self>,
    ) {
        self.update_term(state, req.term, o);

        let res = if req.term < state.current_term {
            request_vote::Response { term: state.current_term, granted: false }
        } else {
            match state.voted_for {
                None => {
                    state.to_mut().voted_for = Some(src);
                    request_vote::Response { term: state.current_term, granted: true }
                }
                Some(id) if id == src => request_vote::Response { term: state.current_term, granted: true },
                Some(_) => request_vote::Response { term: state.current_term, granted: false },
            }
        };

        o.send(src, RaftMsg::RequestVoteRes(res));
    }

    pub fn handle_request_vote_response(
        &self,
        state: &mut Cow<RaftState>,
        src: Id,
        res: request_vote::Response,
        o: &mut Out<Self>,
    ) {
        self.update_term(state, res.term, o);

        if res.term < state.current_term {
            // stale response
        } else if let State::Candidate(candidate) = &state.state {
            if res.granted && !candidate.votes.contains(&src) {
                let mut state = state.to_mut();

                if let State::Candidate(candidate) = &mut state.state {
                    candidate.votes.insert(src);

                    let n_granted = candidate.votes.len() + 1; // count self vote

                    if n_granted >= majority(self.peers.len() + 1) {
                        o.cancel_timer(RaftTimer::Election);

                        state.state = State::Leader(leader_state::State::default());

                        let req = append_entries::Request { term: state.current_term };
                        let req = RaftMsg::AppendEntriesReq(req);
                        o.broadcast(&self.peers, &req);

                        o.set_timer(RaftTimer::Heartbeat, self.config.heartbeat_period..self.config.heartbeat_period);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// https://developer.hashicorp.com/consul/docs/architecture/consensus#deployment_table
    ///
    /// | Servers | Quorum Size | Failure Tolerance |
    /// |:-------:|:-----------:|:-----------------:|
    /// |    1    |      1      |         0         |
    /// |    2    |      2      |         0         |
    /// |    3    |      2      |         1         |
    /// |    4    |      3      |         1         |
    /// |    5    |      3      |         2         |
    /// |    6    |      4      |         2         |
    /// |    7    |      4      |         3         |
    #[test]
    fn test_quorum_size_formula() {
        println!("Servers | Quorum Size");
        for n_servers in 1..=7 {
            println!("{} | {}", n_servers, majority(n_servers));
        }
    }
}
