#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RaftMsg {
    RequestVoteReq(request_vote::Request),
    RequestVoteRes(request_vote::Response),
    AppendEntriesReq(append_entries::Request),
    AppendEntriesRes(append_entries::Response),
}

pub mod request_vote {
    use crate::types::Term;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Request {
        pub term: Term,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Response {
        pub term: Term,
        pub granted: bool,
    }
}

pub mod append_entries {
    use crate::types::Term;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Request {
        pub term: Term,
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    pub struct Response {
        pub term: Term,
        pub success: bool,
    }
}
