use crate::{functions::init_logger, INSTANCE};

use super::prompt;

pub fn instance_or_exit() -> String {
    match INSTANCE.get() {
        Some(i) => i.to_string(),
        None => {
            init_logger(log::LevelFilter::Info);
            prompt("Enter instance address")
        }
    }
}
