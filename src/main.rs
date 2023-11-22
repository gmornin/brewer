use log::*;
use std::error::Error;

use brewer::{
    commands::{core::Clean, TopLevel, TopLevelSubcommands},
    functions::init_logger,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: TopLevel = argp::parse_args_or_exit(argp::DEFAULT);
    trace!("Argument parse success.");

    if args.verbose {
        init_logger(log::LevelFilter::Trace)
    } else {
        init_logger(log::LevelFilter::Info)
    }
    brewer::load()?;

    if !matches!(args.subcommand, TopLevelSubcommands::Clean(_)) {
        tokio::task::spawn(async {
            let _ = Clean::clean_cache().await;
        });
    }

    trace!("Running command {args:?}");

    if args.run().await.is_err() && !args.verbose {
        error!("Command exited unsuccessfully, run with `-v` for verbose debug info.");
    }

    Ok(())
}
