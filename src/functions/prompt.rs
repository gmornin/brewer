use log::*;
use std::io::{stdin, stdout, Write};

use crate::{exit_codes::doas_failed, YES};

pub fn prompt(msg: &str) -> String {
    print!("{msg}:\n> ");
    stdout().flush().unwrap();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("could not read input");
    trace!("Value input recieved.");
    s.trim().to_string()
}

pub fn doasisay(msg: &str) {
    if *YES.get().unwrap() {
        return;
    }

    if prompt(&format!("You are about to carry out `{msg}`.\nIf you understand that this is a potentially dangerous action and wish to proceed,\ntype \"Yes, do as I say\" below")).to_lowercase().as_str() != "yes, do as i say" {
        doas_failed()
    }
}
