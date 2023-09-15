use std::io::{stdin, stdout, Write};

pub fn prompt(msg: &str) -> String {
    print!("{msg}:\n> ");
    stdout().flush().unwrap();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("could not read input");
    s
}
