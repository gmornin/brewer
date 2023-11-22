use std::{error::Error, path::Path, sync::OnceLock, time::UNIX_EPOCH};

use argp::FromArgs;
use chrono::Utc;
use command_macro::CommandTrait;
use log::*;
use tokio::fs::{self, DirEntry};

use crate::{functions::filesize, MAX_AGE};

static mut DELETED_SIZE: OnceLock<u64> = OnceLock::new();
static mut DELETED_COUNT: OnceLock<u64> = OnceLock::new();

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs, Default)]
#[argp(subcommand, name = "clean")]
/// Cleans cache.
pub struct Clean {
    /// Delete all cache even if it is still fresh
    #[argp(switch, short = 'a')]
    pub all: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Clean {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let cache = dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME"));
        if !cache.exists() {
            println!("Nothing in cache, skipping.");
            return Ok(());
        }

        if self.all {
            trace!("Deleting `{}`", cache.to_string_lossy());
            fs::remove_dir_all(&cache).await?;
            println!("Deleted `{}`.", cache.to_string_lossy());
            return Ok(());
        }


        Self::clean_cache().await?;

        println!(
            "{} files deleted, freed {} of disk space.",
            unsafe { DELETED_COUNT.get().unwrap() },
            filesize(unsafe { *DELETED_SIZE.get().unwrap() })
        );

        Ok(())
    }
}

impl Clean {
    pub async fn clean_cache() -> Result<(), Box<dyn Error>> {
        unsafe {
            DELETED_SIZE.set(0).unwrap();
            DELETED_COUNT.set(0).unwrap()
        };

        let cache = dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME"));
        if !fs::try_exists(&cache).await? {
            return Ok(());
        }

        Self::clean(&cache).await
    }

    #[async_recursion::async_recursion]
    async fn clean(path: &Path) -> Result<(), Box<dyn Error>> {
        trace!("Reading directory `{}`", path.to_string_lossy());
        let mut diritems = fs::read_dir(path).await?;
        let mut tasks = Vec::new();

        while let Some(entry) = diritems.next_entry().await? {
            tasks.push(Self::clean_entry(entry));
        }

        for task in tasks {
            task.await?;
        }

        if fs::read_dir(path).await?.next_entry().await?.is_none() {
            fs::remove_dir(path).await?;
        }

        Ok(())
    }

    async fn clean_entry(entry: DirEntry) -> Result<(), Box<dyn Error>> {
        let metadata = entry.metadata().await?;

        if metadata.is_dir() {
            Self::clean(&entry.path()).await?;
        } else if Utc::now().timestamp() as u64
            - metadata.modified()?.duration_since(UNIX_EPOCH)?.as_secs() as u64
            > *MAX_AGE.get().unwrap()
        {
            unsafe {
                *DELETED_SIZE.get_mut().unwrap() += metadata.len();
                *DELETED_COUNT.get_mut().unwrap() += 1;
            }
            trace!("Deleted stale file `{}`", entry.path().to_string_lossy());
            fs::remove_file(entry.path()).await?;
        }

        Ok(())
    }
}
