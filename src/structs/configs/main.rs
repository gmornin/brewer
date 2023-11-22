use config_macro_derive::Config;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(DefaultFromSerde, Serialize, Deserialize, Clone, Config, Debug)]
pub struct MainConfig {
    /// Number of retries when downloading a file.
    #[serde_inline_default(1)]
    #[serde(rename = "download-retries")]
    pub download_retries: u16,
    /// Time before cached item is considered stale and a refetch is needed.
    #[serde_inline_default(3600)]
    #[serde(rename = "max-age")]
    pub max_age: u64,
    /// Automatically runs clean when running commands.
    #[serde_inline_default(true)]
    #[serde(rename = "auto-clean")]
    pub auto_clean: bool,
}
