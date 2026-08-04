# Lab 02 — Wallets and addresses

## Commands used

TODO: Record how you created and inspected both wallets and addresses.

## Terminal output

TODO: Include loaded wallets, addresses, and ownership evidence.
/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1$ cargo test --test lab_02
   Compiling rfb-labs-week-1 v0.1.0 (/home/jemiah/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/lab_02.rs (target/debug/deps/lab_02-a0ccd6ebfff2138b)

running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test verifies_wallet_owns_address ... ok
test lists_loaded_wallets ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

**Wallet context and `-rpcwallet`:**

A Bitcoin Core node can run multiple wallets at once (like `miner` and `receiver` in this lab), each completely separate — different addresses, different balances, different keys.

`-rpcwallet=<name>` tells `bitcoin-cli` which wallet to run a command against. Without it, the node wouldn't know which wallet's addresses or balance you're asking about.

In the Rust code, this is the same idea as passing `Some(wallet_name)` into `client.call(...)` — it just says "do this for this specific wallet."
