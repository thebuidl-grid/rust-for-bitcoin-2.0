# Lab 02 — Wallets and addresses

## Commands used

TODO: Record how you created and inspected both wallets and addresses.
```bash
# 1. Create the 'miner' and 'receiver' wallets
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"

# 2. List all loaded wallets on the node
bitcoin-cli -regtest listwallets

# 3. Generate a labelled address from the 'miner' wallet
bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"

# 4. Generate a labelled address from the 'receiver' wallet
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"

# 5. Verify address ownership in the respective wallets
bitcoin-cli -regtest -rpcwallet=miner getaddressinfo "<MINER_ADDRESS_HERE>"
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo "<RECEIVER_ADDRESS_HERE>"

# 6. Run automated test suite
cargo test --test lab_02
```
## Terminal output

TODO: Include loaded wallets, addresses, and ownership evidence.
elsuraj@El-suraj:~/rust-for-bitcoin-2.0/rfb_labs_week_1$ cargo test --test lab_02
   Compiling rfb-labs-week-1 v0.1.0 (/home/elsuraj/rust-for-bitcoin-2.0/rfb_labs_week_1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.42s
     Running tests/lab_02.rs (target/debug/deps/lab_02-72150c0766a64e66)

running 4 tests
test creates_wallet ... ok
test lists_loaded_wallets ... ok
test generates_labelled_address_in_wallet_context ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

TODO: Link screenshots or describe the attached evidence.
1. Miner Wallet Address: bcrt1q5umafspjxsua3y2g046hhehhueva5zkxc4ywvt (Native SegWit / Bech32 prefix on regtest)

2. Receiver Wallet Address: bcrt1q3daafkaleu04q3p5tdue89rl639u339dndjvln (Native SegWit / Bech32 prefix on regtest)

3. Wallet Loading Check: listwallets returns ["miner", "receiver"].

4. Ownership Verification: getaddressinfo returns "ismine": true when executed against the correct wallet context.

## Explanation

TODO: Explain wallet context and the purpose of `-rpcwallet`.

Why Wallet-Scoped Calls Need -rpcwallet
Bitcoin Core allows multi-wallet operation within a single node process (bitcoind). Because each wallet maintains its own distinct HD seed, key database, UTXO set, and transaction history, commands that create keys or query balances (e.g., getnewaddress, getbalance, getaddressinfo) require explicit context. The -rpcwallet=<wallet_name> parameter (or passing Some(wallet_name) in RPC calls) routes the request to the correct internal wallet instance.

Consequences of Wrong Wallet Context
If a wallet-scoped command is executed without specifying -rpcwallet when multiple wallets are loaded, Bitcoin Core returns an RPC error indicating that the target wallet is ambiguous or not selected. If a command is directed at the wrong wallet context (e.g., querying an address in receiver using -rpcwallet=miner), getaddressinfo will return "ismine": false because the public key does not exist in that wallet's key database.
