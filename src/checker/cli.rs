use clap::{Args, Parser, Subcommand, ValueEnum};
use stateright::actor::{ActorModel, LossyNetwork, Network};

use crate::checker::cfg::RaftModelCfg;
use crate::server::RaftServer;
use crate::types::Term;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Explore(CommandArgs),
    Check(CommandArgs),
}

#[derive(Args)]
pub struct CommandArgs {
    #[arg(long)]
    server_count: usize,

    #[arg(long)]
    network: NetworkArg,

    #[arg(long)]
    lossy_network: bool,

    #[arg(long)]
    max_term: Term,

    #[arg(long)]
    max_crashes: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum NetworkArg {
    Ordered,
    UnorderedDuplicating,
    UnorderedNonDuplicating,
}

impl CommandArgs {
    pub fn into_model(self) -> ActorModel<RaftServer, RaftModelCfg, ()> {
        if self.server_count < 2 * self.max_crashes + 1 {
            panic!(
                "max crashes is {}, but server count is {} (should be >= {})",
                self.max_crashes,
                self.server_count,
                2 * self.max_crashes + 1
            );
        }
        RaftModelCfg {
            server_count: self.server_count,
            network: match self.network {
                NetworkArg::Ordered => Network::new_ordered([]),
                NetworkArg::UnorderedDuplicating => Network::new_unordered_duplicating([]),
                NetworkArg::UnorderedNonDuplicating => Network::new_unordered_nonduplicating([]),
            },
            lossy_network: if self.lossy_network { LossyNetwork::Yes } else { LossyNetwork::No },
            max_term: self.max_term,
            max_crashes: self.max_crashes,
        }
        .into_model()
    }
}
