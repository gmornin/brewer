use log::*;
use std::io::{stdin, stdout, Write};

use crate::{exit_codes::doas_failed, YES};

pub async fn prompt(msg: &str) -> String {
    let msg = msg.to_string();
    tokio::runtime::Runtime::new()
        .unwrap()
        .spawn_blocking(move || prompt_sync(&msg))
        .await
        .unwrap()
}

pub fn prompt_sync(msg: &str) -> String {
    print!("{msg}:\n> ");
    stdout().flush().unwrap();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("could not read input");
    trace!("Value input recieved.");
    s.trim().to_string()
}

pub async fn doasisay(msg: &str) {
    if *YES.get().unwrap() {
        return;
    }

    if prompt(&format!("You are about to carry out `{msg}`.\nIf you understand that this is a potentially dangerous action and wish to proceed,\ntype \"Yes, do as I say\" below")).await.to_lowercase().as_str() != "yes, do as i say" {
        doas_failed()
    }
}
