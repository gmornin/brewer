use log::*;
use std::error::Error;

use brewer::{commands::TopLevel, functions::init_logger};

fn main() -> Result<(), Box<dyn Error>> {
    let args: TopLevel = argp::parse_args_or_exit(argp::DEFAULT);
    trace!("Argument parse success.");

    if args.verbose {
        init_logger(log::LevelFilter::Trace)
    } else {
        init_logger(log::LevelFilter::Info)
    }
    brewer::load()?;

    trace!("Running command {args:?}");

    if args.run().is_err() && !args.verbose {
        error!("Command exited unsuccessfully, run with `-v` for verbose debug info.");
    }

    Ok(())
}
