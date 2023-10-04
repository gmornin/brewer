use std::{error::Error, path::PathBuf, sync::OnceLock};

use config_macro::ConfigTrait;
use log::*;

use crate::structs::{CredsConfig, GmIgnoreDefault, MainConfig};

pub static HTTP: OnceLock<bool> = OnceLock::new();
pub static YES: OnceLock<bool> = OnceLock::new();
pub static mut CREDS: OnceLock<CredsConfig> = OnceLock::new();
pub static mut INSTANCE: OnceLock<String> = OnceLock::new();
pub static BASE_PATH: OnceLock<String> = OnceLock::new();
pub static OUTPUT_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static DOWNLOAD_RETRIES: OnceLock<u16> = OnceLock::new();
pub static GMIGNORE_DEFAULT: OnceLock<String> = OnceLock::new();
pub const EXPECT: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8";

pub mod exit_codes {
    use std::{error::Error, path::Path, process};

    use log::error;

    // 300s: operation not allowed
    //
    /// This operation is only allowed when not logged in.
    pub fn loggedin_not_allowed() {
        error!("3000 This operation is not allowed when logged in.");
        process::exit(3000)
    }
    pub fn loggedin_only() {
        error!("3001 This operation can only be done when logged in.");
        process::exit(3001)
    }

    // 400s: not found
    //
    /// When an optional argument is missing, but is required.
    pub fn missing_argument(msg: &str) {
        error!("4000 Argument `{msg}` is required but not provided.");
        process::exit(4000)
    }

    pub fn missing_repo_json() {
        error!("4001 Cannot find gmrepo.json, is this a cloned repo?");
        process::exit(4001)
    }

    // 500s: error/aborted
    //
    /// When "do as I say" failed.
    pub fn doas_failed() {
        error!("5000 Aborted: user did not enter confirm message.");
        process::exit(5000)
    }

    /// .ignore file adding failed
    pub fn ignore_add_failed(path: &Path) {
        error!(
            "5001 Aborted: could not add .ignore file at `{}`.",
            path.to_string_lossy().to_string()
        );
        process::exit(5001)
    }

    /// clone url bad first lined json
    pub fn bad_clone_json() {
        error!("5002 Aborted: invalid page first lined JSON in url.");
        process::exit(5002)
    }

    /// bad url format
    pub fn bad_url(msg: &str, url: &str) {
        error!("5003 Invalid url format in {url}: {msg}");
        process::exit(5003)
    }

    /// output path already exists
    pub fn output_path_occupied(path: &Path) {
        error!(
            "5004 Output path `{}` is already occupied.",
            path.to_string_lossy()
        );
        process::exit(5004)
    }

    /// donwload failed
    pub fn download_failed(path: &str) {
        error!("5005 Downloading failed for {path}, aborting.");
        process::exit(5005)
    }

    // push or pull fail
    pub fn sync_failed(e: Box<dyn Error>) {
        error!("5006 Syncing failed with error {e}, aborting.");
        process::exit(5006)
    }

    // failed to create .gmignore
    pub fn create_gmignore_fail(e: Box<dyn Error>, path: &Path) {
        error!(
            "5007 Failed to create .gmignore in path {} with error {e}",
            path.to_string_lossy()
        );
        process::exit(5007)
    }

    /// there is a conflict between remote and local
    pub fn repo_conflict() {
        error!("5008 Aborted action as there is a conflict between local and remote.");
        process::exit(5008)
    }
}

pub fn load() -> Result<(), Box<dyn Error>> {
    debug!("Loading main config from {:?}", MainConfig::path());
    let main = MainConfig::load()?;
    trace!("Main config loaded and parsed.");
    DOWNLOAD_RETRIES.set(main.download_retries).unwrap();

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

    GMIGNORE_DEFAULT
        .set(GmIgnoreDefault::load().unwrap())
        .unwrap();

    Ok(())
}
