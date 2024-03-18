use config_macro_derive::Config;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Config, Debug)]
pub struct CredsConfig {
    pub id: i64,
    pub instance: String,
    pub token: String,
}

impl CredsConfig {
    pub fn is_loggedin(&self) -> bool {
        self.id != 0
    }

    pub fn redact(&self) -> Self {
        Self {
            token: "redacted".to_string(),
            ..self.clone()
        }
    }
}
