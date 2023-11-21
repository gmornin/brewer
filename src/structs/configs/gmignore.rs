use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

use log::*;

use crate::{exit_codes::create_gmignore_fail, GMIGNORE_DEFAULT};

const DEFAULT: &str = r#".git
.gmrepo.json"#;

pub struct GmIgnoreDefault;

impl GmIgnoreDefault {
    pub fn load() -> Result<String, Box<dyn Error>> {
        let path = dirs::config_dir()
            .unwrap()
            .join(config_macro::NAME)
            .join("gmignore.default");
        if path.exists() {
            debug!("GM ignore default exists, reading file.");
            return Ok(fs::read_to_string(&path)?);
        }
        debug!("No GM ignore default found at {:?}, using default", path);
        trace!("Saving GM ignore default file after load to {:?}", path);
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)?;
        file.write_all(DEFAULT.as_bytes())?;
        Ok(DEFAULT.to_string())
    }

    pub fn create(path: &Path) {
        let path = path.join(".gmignore");
        if let Err(e) = (|| -> Result<(), Box<dyn Error>> {
            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&path)?;
            file.write_all(GMIGNORE_DEFAULT.get().unwrap().as_bytes())?;
            Ok(())
        })() {
            create_gmignore_fail(e, &path);
        }
    }
}
