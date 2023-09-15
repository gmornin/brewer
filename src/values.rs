use std::{error::Error, sync::OnceLock};

use config_macro::ConfigTrait;

use crate::structs::CredsConfig;

pub static HTTP: OnceLock<bool> = OnceLock::new();
pub static mut CREDS: OnceLock<CredsConfig> = OnceLock::new();
pub static INSTANCE: OnceLock<String> = OnceLock::new();

pub mod exit_codes {
    use std::process;

    use log::error;

    // 300s: operation not allowed
    //
    /// This operation is only allowed when not logged in.
    pub fn loggedin_not_allowed() {
        error!("This operation is not allowed when logged in.");
        process::exit(30000)
    }

    // 400s: not found
    //
    /// When an optional argument is missing, but is required.
    pub fn missing_argument(msg: &str) {
        error!("Argument `{msg}` is required but not provided.");
        process::exit(40000)
    }
}

pub fn load() -> Result<(), Box<dyn Error>> {
    unsafe {
        CREDS.set(CredsConfig::load()?).unwrap();
    }

    Ok(())
}
