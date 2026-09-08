//! Manual runner for Lab 07 against a live Polar regtest node.
//!
//! Note: the confirming block was already mined while gathering Lab 06
//! evidence (see submissions/lab_06.md for why), so this demo calls the
//! granular Lab 07 functions against that already-confirmed state instead of
//! `confirm_and_locate_transaction` (which would mine a second block and
//! push the transaction to 2 confirmations). `confirm_and_locate_transaction`
//! itself is exercised end-to-end by `cargo test --test lab_07` with mocks.
//!
//! Usage: BITCOIN_CLI=/path/to/wrapper cargo run --example lab07_demo

use rfb_labs_week_1::labs::lab07_confirm::{mempool_is_empty, transaction_confirmations};
use rfb_labs_week_1::rpc::{ProcessRpc, RpcClient};
use serde_json::Value;

const TXID: &str = "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe";

fn main() {
    let binary = std::env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_string());
    let rpc = ProcessRpc::new(binary);

    let empty = mempool_is_empty(&rpc).expect("getrawmempool failed");
    println!("mempool_is_empty = {empty}");

    let confirmations =
        transaction_confirmations(&rpc, "receiver", TXID).expect("gettransaction failed");
    println!("confirmations    = {confirmations}");

    let raw = rpc
        .call(Some("receiver"), "gettransaction", &[TXID.to_string()])
        .expect("gettransaction failed");
    let value: Value = serde_json::from_str(&raw).unwrap();
    let block_hash = value["blockhash"].as_str().unwrap().to_string();
    println!("block_hash       = {block_hash}");

    let block_raw = rpc
        .call(None, "getblock", &[block_hash.clone(), "1".to_string()])
        .expect("getblock failed");
    let block_value: Value = serde_json::from_str(&block_raw).unwrap();
    let contains = block_value["tx"]
        .as_array()
        .unwrap()
        .iter()
        .any(|t| t.as_str() == Some(TXID));
    println!("block contains txid? = {contains}");
}
