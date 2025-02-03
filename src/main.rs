use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use self::cli::{Cli, Cmd};

mod cli;

fn main() {
    FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_env_var("DUMB_AUTH_LOG")
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = Cli::parse();
    match cli.cmd {
        None => cli::run(cli.args.unwrap()),
        Some(Cmd::Passwd(args)) => cli::passwd(args),
    };
}
