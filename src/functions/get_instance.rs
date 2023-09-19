use crate::INSTANCE;

use log::*;

use super::prompt;

pub fn get_instance() -> String {
    match unsafe { INSTANCE.get() } {
        Some(i) if !i.is_empty() => {
            trace!("Instance already contains value, skipping.");
            i.to_string()
        }
        Some(_) => {
            debug!("Instance contains empty string, prompting for new value.");
            let i = prompt("Enter instance address");
            *unsafe { INSTANCE.get_mut().unwrap() } = i.trim().to_string();
            i
        }
        None => {
            debug!("Instance is empty, prompting for new value.");
            let i = prompt("Enter instance address");
            *unsafe { INSTANCE.get_mut().unwrap() } = i.trim().to_string();
            i
        }
    }
}
