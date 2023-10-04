use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use command_macro_derive::Command;

use crate::*;

use self::core::*;

pub mod core;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
/// CLI for GM services.
pub struct TopLevel {
    /// Show all workings.
    #[argp(switch, short = 'v', global)]
    pub verbose: bool,
    /// Yes, do as I say.
    #[argp(switch, short = 'y', global)]
    pub yes: bool,
    /// Use unencrypted http traffic instead of https.
    #[argp(switch, global)]
    pub http: bool,

    #[argp(subcommand)]
    pub subcommand: TopLevelSubcommands,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs, Command)]
#[argp(subcommand)]
pub enum TopLevelSubcommands {
    Register(Register),
    Version(Version),
    Login(Login),
    Logout(Logout),
    Regen(Regen),
    Delete(Delete),
    Status(Status),
    Rename(Rename),
    Passwd(Passwd),
    Email(Email),
    Verify(Verify),

    Ls(Ls),
    Mv(Mv),
    Cp(Cp),
    Touch(Touch),
    Rm(Rm),
    Exist(Exist),
    Mkdir(Mkdir),
    Vis(Vis),

    Clone(Clone),
    Pull(Pull),
}

impl TopLevel {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        HTTP.set(self.http).unwrap();
        YES.set(self.yes).unwrap();

        self.subcommand.run()?;

        Ok(())
    }
}
