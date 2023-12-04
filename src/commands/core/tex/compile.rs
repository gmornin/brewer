use std::{error::Error, ffi::OsStr, path::PathBuf};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{Compiler, FromFormat, ToFormat, V1Compile, V1Response};
use log::*;

use crate::{
    exit_codes::{loggedin_only, missing_argument, unknown_compiler, unknown_format},
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "compile")]
/// Compiles a remote file.
pub struct Compile {
    #[argp(positional)]
    /// The path of source file (omit beginning `/tex`).
    pub path: String,
    #[argp(option, short = 'f')]
    /// Format to compile from (inferred from file extension if left empty).
    pub from: Option<String>,
    #[argp(option, short = 't')]
    /// Format to compile to (inferred from file extension if left empty).
    pub to: Option<String>,
    #[argp(option, short = 'c')]
    /// Compiler used for compiling (uses default compiler if left empty).
    pub compiler: Option<String>,
}

#[async_trait::async_trait]
impl CommandTrait for Compile {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with compiling file.");

        let from = match self.from.as_ref() {
            None => match PathBuf::from(&self.path)
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap()
            {
                "md" => FromFormat::Markdown,
                "tex" => FromFormat::Latex,
                _ => {
                    missing_argument("from");
                    unreachable!()
                }
            },
            Some(s) => match s.as_str() {
                "markdown" | "md" => FromFormat::Markdown,
                "tex" | "lt" | "latex" => FromFormat::Latex,
                _ => {
                    unknown_format(s);
                    unreachable!()
                }
            },
        };

        let to = match self.to.as_ref() {
            None => match PathBuf::from(&self.path)
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap()
            {
                "md" => ToFormat::Html,
                "tex" => ToFormat::Pdf,
                _ => {
                    missing_argument("to");
                    unreachable!()
                }
            },
            Some(s) => match s.as_str() {
                "html" => ToFormat::Html,
                "pdf" => ToFormat::Pdf,
                _ => {
                    unknown_format(s);
                    unreachable!()
                }
            },
        };

        let compiler = self.compiler.as_ref().map(|s| match s.as_str() {
            "pulldown cmark" | "cmark" => Compiler::PulldownCmark,
            "pdflatex" => Compiler::Pdflatex,
            s => {
                unknown_compiler(s);
                unreachable!()
            }
        });

        let path = self.path.trim_matches('/');
        let body = V1Compile {
            from,
            to,
            compiler,
            token: creds.token.clone(),
            path: path.to_string(),
        };

        println!("Running compile task...");
        let url = get_url("/api/compile/v1/simple").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
