use std::error::Error;

use brewer::commands::TopLevel;

fn main() -> Result<(), Box<dyn Error>> {
    let args: TopLevel = argp::parse_args_or_exit(argp::DEFAULT);
    brewer::load()?;

    args.run().unwrap();

    Ok(())
}
