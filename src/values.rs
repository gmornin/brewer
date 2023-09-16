use std::{error::Error, sync::OnceLock};

use config_macro::ConfigTrait;
use log::*;

use crate::structs::CredsConfig;

pub static HTTP: OnceLock<bool> = OnceLock::new();
pub static mut CREDS: OnceLock<CredsConfig> = OnceLock::new();
pub static mut INSTANCE: OnceLock<String> = OnceLock::new();
pub static BASE_PATH: OnceLock<String> = OnceLock::new();

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
    pub fn loggedin_only() {
        error!("This operation can only be done when logged in.");
        process::exit(30001)
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
    debug!("Loading creds config from {:?}", CredsConfig::path());
    let creds = CredsConfig::load()?;
    trace!("Creds config loaded and parsed.");
    unsafe {
        if creds.is_loggedin() {
            trace!(
                "Creds indicates account is logged in, setting instance to {}",
                creds.instance
            );
            INSTANCE.set(creds.instance.clone()).unwrap();
        }
        trace!("Settind `CREDS` to {creds:?}");
        CREDS.set(creds).unwrap();
    }

    Ok(())
}
