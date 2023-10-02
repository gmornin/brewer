use config_macro_derive::Config;
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(DefaultFromSerde, Serialize, Deserialize, Clone, Config, Debug)]
pub struct MainConfig {
    #[serde_inline_default(3)]
    #[serde(rename = "download-retries")]
    pub download_retries: u16,
}
