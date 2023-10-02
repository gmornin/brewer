use std::{
    fmt::Display,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use log::*;
use serde::{de::DeserializeOwned, Serialize};

pub const NAME: &str = "brewer";

pub trait ConfigTrait
where
    Self: Serialize + DeserializeOwned + Clone + Default,
{
    const NAME: &'static str;

    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join(NAME)
            .join(format!("{}.yml", Self::NAME))
    }

    fn load() -> Result<Self, ConfigError> {
        create_parent()?;
        let path = Self::path();

        debug!("Reading config file at {:?}", Self::path());

        let config = if path.exists() {
            debug!("Config file exists, reading file.");
            let s = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    return Err(ConfigError::FsError { path, error: e });
                }
            };

            trace!("Deserializing file content.");
            match serde_yaml::from_str(&s) {
                Ok(v) => v,
                Err(e) => {
                    return Err(ConfigError::ParseError { path, error: e });
                }
            }
        } else {
            debug!("No config file found at {:?}, using default", Self::path());
            Self::default()
        };

        trace!("Saving config file after load to {:?}", Self::path());
        config.save()?;
        Ok(config)
    }

    fn save(&self) -> Result<(), ConfigError> {
        let s = serde_yaml::to_string(&self).unwrap();
        let path = Self::path();
        trace!("Saving config file to {:?}", path);
        let mut file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => return Err(ConfigError::FsError { path, error: e }),
        };
        if let Err(e) = file.write_all(s.as_bytes()) {
            return Err(ConfigError::FsError { path, error: e });
        }
        trace!("Config file saved.");

        Ok(())
    }

    fn clear(&mut self) {
        *self = Self::default()
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FsError {
        path: PathBuf,
        error: io::Error,
    },
    ParseError {
        path: PathBuf,
        error: serde_yaml::Error,
    },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FsError { path, error } => f.write_fmt(format_args!(
                "error reading config file at {path:?}: {error}"
            )),
            Self::ParseError { path, error } => f.write_fmt(format_args!(
                "error parsing config file at {path:?}: {error}"
            )),
        }
    }
}

impl std::error::Error for ConfigError {}

fn create_parent() -> Result<(), ConfigError> {
    let path = dirs::config_dir().unwrap().join(NAME);
    if path.exists() {
        return Ok(());
    }
    debug!("Directory does not exist in {:?}, creating.", path);
    fs::create_dir(&path).map_err(move |e| ConfigError::FsError { path, error: e })
}
