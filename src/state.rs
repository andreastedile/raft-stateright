use stateright::actor::Id;

use crate::types::Term;

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct RaftState {
    pub current_term: Term,
    pub voted_for: Option<Id>,
    pub state: State,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub enum State {
    Follower,
    Candidate(candidate_state::State),
    Leader(leader_state::State),
}

pub mod candidate_state {
    use stateright::actor::Id;
    use stateright::util::HashableHashSet;

    #[derive(Clone, Debug, PartialEq, Hash)]
    pub struct State {
        pub votes: HashableHashSet<Id>,
        pub n_consecutive_timeouts: usize,
    }

    impl Default for State {
        fn default() -> Self {
            State { votes: Default::default(), n_consecutive_timeouts: 0 }
        }
    }
}

pub mod leader_state {
    #[derive(Clone, Debug, PartialEq, Hash)]
    pub struct State {
        pub n_consecutive_timeouts: usize,
    }

    impl Default for State {
        fn default() -> Self {
            State { n_consecutive_timeouts: 0 }
        }
    }
}
