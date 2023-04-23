use clap::Parser;
use raft_stateright::checker::cli::{Cli, Commands};
use stateright::report::WriteReporter;
use stateright::{Checker, Model};

pub fn main() {
    match Cli::parse().command {
        Commands::Explore(args) => {
            args.into_cfg().into_model().checker().threads(num_cpus::get()).serve("localhost:3000");
        }
        Commands::Check(args) => {
            let cfg = args.into_cfg();
            let res = cfg
                .clone()
                .into_model()
                .checker()
                .threads(num_cpus::get())
                .spawn_dfs()
                .join_and_report(&mut WriteReporter::new(&mut std::io::stdout()));

            println!("Statistics:\n{}", cfg.stats.lock().unwrap());

            res.assert_properties();
        }
    };
}
