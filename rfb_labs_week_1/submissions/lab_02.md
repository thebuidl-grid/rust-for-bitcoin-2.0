# Lab 02 — Wallet creation and address management

## Commands used

Commands executed:

```bash
cargo test --test lab_02

Bitcoin Core RPC calls used by the lab:

createwallet
listwallets
getnewaddress
getaddressinfo
Terminal output

The Lab 02 test suite completed successfully.

Finished test profile [unoptimized + debuginfo] target(s) in 0.08s
Running tests\lab_02.rs

running 4 tests

test generates_labelled_address_in_wallet_context ... ok
test creates_wallet ... ok
test verifies_wallet_owns_address ... ok
test lists_loaded_wallets ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

The tests confirmed that the Rust application successfully communicated with the Bitcoin Core regtest node and was able to:

Create wallets.
List loaded wallets.
Generate wallet addresses.
Verify address ownership information.
Evidence references

Evidence collected:

Terminal output showing cargo test --test lab_02 passing.
Screenshot reference: lab02-test-success.png.
Explanation

This lab demonstrates Bitcoin Core wallet management through Rust RPC communication.

The implementation creates wallets, generates addresses, lists available wallets, and verifies ownership information for generated addresses.

Bitcoin Core wallets provide a controlled environment for managing keys and transaction-related operations. The Rust application communicates with Bitcoin Core using RPC commands to perform wallet actions safely on the regtest network.

The successful tests confirm that wallet creation, address generation, wallet inspection, and ownership verification work correctly.