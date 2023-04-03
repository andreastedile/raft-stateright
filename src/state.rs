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
    Leader,
}

pub mod candidate_state {
    use stateright::actor::Id;
    use stateright::util::HashableHashSet;

    #[derive(Clone, Debug, PartialEq, Hash)]
    pub struct State {
        pub votes: HashableHashSet<Id>,
    }

    impl Default for State {
        fn default() -> Self {
            State { votes: Default::default() }
        }
    }
}
