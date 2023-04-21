use stateright::actor::Id;
use stateright::util::HashableHashSet;

use crate::types::Term;

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum RaftState {
    Follower { current_term: Term, voted_for: Option<Id> },
    Candidate { current_term: Term, voted_for: Option<Id>, votes: HashableHashSet<Id> },
    Leader { current_term: Term, voted_for: Option<Id> },
}

impl RaftState {
    pub fn current_term(&self) -> Term {
        match self {
            RaftState::Follower { current_term, .. } => *current_term,
            RaftState::Candidate { current_term, .. } => *current_term,
            RaftState::Leader { current_term, .. } => *current_term,
        }
    }

    pub fn voted_for(&self) -> Option<Id> {
        match self {
            RaftState::Follower { voted_for, .. } => *voted_for,
            RaftState::Candidate { voted_for, .. } => *voted_for,
            RaftState::Leader { voted_for, .. } => *voted_for,
        }
    }
}
