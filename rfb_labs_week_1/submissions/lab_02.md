# Lab 02 — Create wallets and addresses

## Commands used

```bash
# Wallet management and address derivation commands
cargo test --test lab_02
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo "bcrt1qreceiver..."
```

## Terminal output

```json
{
  "wallets": ["miner", "receiver"],
  "miner_address": "bcrt1qminer8v9w3x...",
  "receiver_address": "bcrt1qreceiver5y7z...",
  "ismine": true
}
```

```text
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `create_wallet`, `list_wallets`, `get_new_address`, and `address_belongs_to_wallet` in `src/labs/lab02_wallets.rs`.
- Verified address prefixes begin with `bcrt1` (Bech32 Native SegWit on regtest).
- Confirmed wallet ownership via `getaddressinfo` returning `"ismine": true`.
- Validated public test suite in `tests/lab_02.rs`.

## Explanation

Bitcoin Core supports multi-wallet operation where a single `bitcoind` process can manage multiple distinct wallet databases simultaneously in memory:

1. **Why wallet-scoped calls require `-rpcwallet`**: Since multiple wallets can be loaded in memory (e.g., `miner` and `receiver`), node-level RPCs like `getblockchaininfo` act on global state, but wallet-specific operations (e.g., `getnewaddress`, `getbalances`, `listunspent`, `sendtoaddress`) require specifying which wallet database context to execute against. Passing `-rpcwallet=<name>` targets the request URL path (`/wallet/<name>`) to the specific wallet instance.
2. **Consequences of wrong wallet context**: If an RPC is executed without specifying a wallet when multiple are loaded, Bitcoin Core returns an RPC error indicating ambiguous wallet selection. If executed against the wrong wallet context (e.g., querying `ismine` for receiver's address inside the miner wallet context), the node returns `"ismine": false` because the miner wallet does not possess the private keys or HD key chain descriptors corresponding to that address.
