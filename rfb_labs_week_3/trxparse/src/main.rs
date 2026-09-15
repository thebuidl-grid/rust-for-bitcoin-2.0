use std::io::{IsTerminal, Read};
use trxparse::parse_transaction;

/// Default sample: a mainnet P2WPKH SegWit transaction, used when no argument
/// is supplied so the program is runnable with a bare `cargo run`.
const SAMPLE: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

fn main() {
    let raw = match std::env::args().nth(1) {
        Some(arg) => arg,
        // Accept a hex string piped in on stdin, so this composes with other
        // tools; fall back to the sample when there is nothing to read.
        None => {
            let mut buffer = String::new();
            if !std::io::stdin().is_terminal() {
                let _ = std::io::stdin().read_to_string(&mut buffer);
            }
            if buffer.trim().is_empty() {
                SAMPLE.to_string()
            } else {
                buffer
            }
        }
    };

    match parse_transaction(&raw) {
        Ok(value) => println!("{}", serde_json::to_string_pretty(&value).unwrap()),
        Err(error) => {
            eprintln!("error: could not parse transaction: {}", error);
            std::process::exit(1);
        }
    }
}
