# Lab 02 — Wallets and addresses

## Commands used

Rust commands, run against the live Polar node (`polar-n1-backend1`,
Bitcoin Core 30.0):
```
cargo test --test lab_02
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab02_demo
```

Bitcoin Core RPCs called by `src/labs/lab02_wallets.rs` (via `bitcoin-cli`):
```
createwallet "miner"
createwallet "receiver"
listwallets
getnewaddress "mining"     -rpcwallet=miner
getnewaddress "classmate"  -rpcwallet=receiver
getaddressinfo <mining address>    -rpcwallet=miner
getaddressinfo <classmate address> -rpcwallet=receiver
getaddressinfo <classmate address> -rpcwallet=miner   # negative-ownership check
```

## Terminal output

`cargo test --test lab_02`:
```
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab02_demo` against the live node:
```
createwallet(miner) -> ok
createwallet(receiver) -> ok
loaded wallets = ["", "miner", "receiver"]
mining address    = bcrt1qj936wq2p5xz50lp8unxma2z0tt82dtqyz4pjtv
classmate address = bcrt1qxmst06mxnlgm5u7tscqsvvf892x8ulsasrl5ua
miner owns mining address?       = true
receiver owns classmate address?  = true
miner owns classmate address?     = false (expected false)
```

## Evidence references

- Screenshot: `submissions/evidence/Screenshot from 2026-08-01 13-57-53.png` — IDE
  terminal running `cargo test --test lab_02`, all 4 tests passing.
- `listwallets` shows `["", "miner", "receiver"]` — both `miner` and `receiver`
  are loaded alongside the node's default `""` wallet, proving both wallets
  exist and are active.
- Both generated addresses start with the `bcrt1` regtest bech32 prefix
  (`bcrt1qj936w...` and `bcrt1qxmst0...`), confirming the node is issuing
  regtest-format addresses, not mainnet (`bc1...`) or testnet (`tb1...`)
  addresses.
- `getaddressinfo` run in the `miner` wallet's context for the mining address
  returns `ismine: true`; the same call in the `receiver` wallet's context for
  the classmate address also returns `true`. Critically, asking the `miner`
  wallet about the `receiver`'s classmate address returns `false` — proving
  address ownership is wallet-scoped, not node-wide.

## Explanation

Bitcoin Core can hold multiple independent wallets loaded at once inside a
single node process. An RPC like `getnewaddress` or `getaddressinfo` doesn't
know which wallet you mean unless you tell it — that's what `-rpcwallet=<name>`
(`Some(wallet_name)` in `RpcClient::call`) is for. Without it, Core either
falls back to a legacy unnamed default wallet or, if none/multiple wallets are
loaded, refuses the call outright.

A "wrong wallet context" means asking a wallet about state it doesn't own —
e.g. calling `getaddressinfo` for the classmate address while scoped to
`miner` instead of `receiver`. Core will happily answer (the address is still
valid on the shared chain), but `ismine` correctly comes back `false`, because
`miner`'s keypool never derived that address. If a program mixed up wallet
context, it could believe it owns an address (or a balance, or a UTXO) that a
completely different wallet actually controls — a dangerous class of bug when
real money is at stake, which is exactly why this lab tests both the positive
(`true`) and negative (`false`) cases explicitly.
