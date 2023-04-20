use clap::Parser;
use raft_stateright::checker::cli::{Cli, Commands};
use stateright::report::WriteReporter;
use stateright::{Checker, Model};

pub fn main() {
    match Cli::parse().command {
        Commands::Explore(args) => {
            args.into_model().checker().threads(num_cpus::get()).serve("localhost:3000");
        }
        Commands::Check(args) => {
            let res = args
                .into_model()
                .checker()
                .threads(num_cpus::get())
                .spawn_dfs()
                .join_and_report(&mut WriteReporter::new(&mut std::io::stdout()));

            res.assert_properties();
        }
    };
}
