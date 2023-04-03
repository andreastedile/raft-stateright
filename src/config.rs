use std::ops::Range;
use std::time::Duration;

pub const ELECTION_TIMEOUT: Range<Duration> = Duration::from_millis(150)..Duration::from_millis(300);

#[derive(Debug)]
pub struct RaftConfig {
    pub election_timeout: Range<Duration>,
    pub heartbeat_period: Duration,
}

impl Default for RaftConfig {
    fn default() -> Self {
        RaftConfig { election_timeout: ELECTION_TIMEOUT, heartbeat_period: ELECTION_TIMEOUT.start / 2 }
    }
}
