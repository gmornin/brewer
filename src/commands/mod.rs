use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use command_macro_derive::Command;

use crate::{functions::init_logger, *};

use self::core::*;

pub mod core;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
/// CLI for GM services.
pub struct TopLevel {
    /// Show all workings.
    #[argp(switch, short = 'v', global)]
    pub verbose: bool,
    /// Use unencrypted http traffic instead of https.
    #[argp(switch, global)]
    pub http: bool,
    // #[argp(option, short = 'i', default = "crate::functions::instance_or_exit()")]
    #[argp(option, short = 'i', default = "String::new()")]
    /// Instance domain or IP.
    pub instance: String,

    #[argp(subcommand)]
    pub subcommand: TopLevelSubcommands,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs, Command)]
#[argp(subcommand)]
pub enum TopLevelSubcommands {
    Create(Create),
    Version(Version),
}

impl TopLevel {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        if self.verbose {
            init_logger(log::LevelFilter::Trace)
        } else {
            init_logger(log::LevelFilter::Info)
        }

        HTTP.set(self.http).unwrap();

        INSTANCE.set(self.instance.clone()).unwrap();
        self.subcommand.run().unwrap();

        Ok(())
    }
}
