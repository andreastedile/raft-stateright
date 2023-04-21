use std::borrow::Cow;

use stateright::actor::{majority, Id, Out};

use crate::messages::{append_entries, request_vote, RaftMsg};
use crate::server::RaftServer;
use crate::state::RaftState;
use crate::timers::RaftTimer;

impl RaftServer {
    pub fn on_election_timeout(&self, state: &mut Cow<RaftState>, o: &mut Out<Self>) {
        match state.as_ref() {
            RaftState::Follower { current_term, .. } => {
                *state.to_mut() = RaftState::Candidate {
                    current_term: current_term + 1,
                    voted_for: Some(self.id),
                    votes: Default::default(),
                };

                let req = request_vote::Request { term: state.current_term() };
                let req = RaftMsg::RequestVoteReq(req);
                o.broadcast(&self.peers, &req);

                o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
            }
            RaftState::Candidate { current_term, voted_for, .. } => {
                *state.to_mut() = RaftState::Candidate {
                    current_term: current_term + 1,
                    voted_for: *voted_for,
                    votes: Default::default(),
                };

                let req = request_vote::Request { term: state.current_term() };
                let req = RaftMsg::RequestVoteReq(req);
                o.broadcast(&self.peers, &req);

                o.set_timer(RaftTimer::Election, self.config.election_timeout.clone());
            }
            RaftState::Leader { .. } => unreachable!(),
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

        let res = if req.term < state.current_term() {
            request_vote::Response { term: state.current_term(), granted: false }
        } else {
            // if req.term was > state.current_term(),
            // then the state was reset to follower and voted_for is None.
            // if req.term was == state.current_term(),
            // then state can be follower (any voted_for case is possible), candidate or leader (wth voted_for == self.id).
            match state.voted_for() {
                None => {
                    if let RaftState::Follower { current_term, .. } = state.as_ref() {
                        *state.to_mut() = RaftState::Follower { current_term: *current_term, voted_for: Some(src) }
                    } else {
                        unreachable!()
                    }

                    request_vote::Response { term: state.current_term(), granted: true }
                }
                Some(id) if id == src => request_vote::Response { term: state.current_term(), granted: true },
                Some(_) => request_vote::Response { term: state.current_term(), granted: false },
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

        if res.term < state.current_term() {
            // stale response
        } else if let RaftState::Candidate { current_term, voted_for, votes } = state.as_ref() {
            if res.granted && !votes.contains(&src) {
                let mut votes = votes.clone();
                votes.insert(src);

                let n_granted = votes.len() + 1; // count self vote

                if n_granted < majority(self.peers.len() + 1) {
                    *state.to_mut() =
                        RaftState::Candidate { current_term: *current_term, voted_for: *voted_for, votes };
                } else {
                    o.cancel_timer(RaftTimer::Election);

                    *state.to_mut() = RaftState::Leader { current_term: *current_term, voted_for: *voted_for };

                    let req = append_entries::Request { term: state.current_term() };
                    let req = RaftMsg::AppendEntriesReq(req);
                    o.broadcast(&self.peers, &req);

                    o.set_timer(RaftTimer::Heartbeat, self.config.heartbeat_period..self.config.heartbeat_period);
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
