#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RaftTimer {
    Election,
    Heartbeat,
}
