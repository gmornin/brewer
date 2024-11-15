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
    Version(Version),
    Clean(Clean),
    Register(Register),
    Login(Login),
    Logout(Logout),
    Regen(Regen),
    Delete(Delete),
    Status(Status),
    Rename(Rename),
    Passwd(Passwd),
    Email(Email),
    Verify(Verify),
    Enable(Enable),
    Allow(Allow),
    Deny(Deny),
    Access(Access),
    AccessTo(AccessTo),
    Invite(Invite),

    Jobs(Jobs),

    Ls(Ls),
    Tree(Tree),
    Mv(Mv),
    Cp(Cp),
    Touch(Touch),
    Rm(Rm),
    Exist(Exist),
    Mkdir(Mkdir),
    Vis(Vis),
    Upload(Upload),
    Open(Open),

    Clone(Clone),
    Pull(Pull),
    Push(Push),
    Bind(Bind),

    Compile(Compile),
    Publish(Publish),
    Publishes(Publishes),

    Render(Render),
}

impl TopLevel {
    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        HTTP.set(self.http).unwrap();
        YES.set(self.yes).unwrap();

        self.subcommand.run().await?;

        Ok(())
    }
}
