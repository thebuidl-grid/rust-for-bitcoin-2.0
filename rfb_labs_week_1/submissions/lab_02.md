# Lab 02 — Wallets and addresses

## Commands used

Rust:

```
cargo test --test lab_02
cargo fmt --check
cargo run --example lab02
```

`examples/lab02.rs` calls the completed `list_wallets`, `get_new_address`, and
`address_belongs_to_wallet` functions against the real node (same `docker exec -u bitcoin
polar-n1-backend1 bitcoin-cli -regtest ...` routing used in Lab 01). It doesn't call
`create_wallet` again since the wallets already existed from the manual step below (calling
`createwallet` on an existing wallet errors).

Bitcoin Core RPCs (run directly in Polar's node terminal):

```
bitcoin-cli createwallet miner
bitcoin-cli createwallet receiver
bitcoin-cli listwallets
bitcoin-cli -rpcwallet=miner getnewaddress mining
bitcoin-cli -rpcwallet=receiver getnewaddress classmate
bitcoin-cli -rpcwallet=miner getaddressinfo <miner address>
bitcoin-cli -rpcwallet=receiver getaddressinfo <receiver address>
bitcoin-cli -rpcwallet=receiver getaddressinfo <miner address>   # wrong-wallet-context check
```

## Terminal output

`cargo test --test lab_02`:

```
running 4 tests
test creates_wallet ... ok
test lists_loaded_wallets ... ok
test generates_labelled_address_in_wallet_context ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

`cargo run --example lab02` (real node, via the completed Rust implementation):

```
Loaded wallets: ["", "miner", "receiver"]
miner address: bcrt1qhs7pchgsklu65vu0vhaq87zga67kqx7edq5fgr
receiver address: bcrt1qr65ta6chmnmrpxsa6qt2mn7xglzxyu04ghs7xx
miner wallet owns its own address: true
receiver wallet owns miner's address: false
```

Raw `bitcoin-cli` output, run directly in Polar's terminal (cross-checking the same node):

```
$ bitcoin-cli createwallet miner
{ "name": "miner" }

$ bitcoin-cli createwallet receiver
{ "name": "receiver" }

$ bitcoin-cli listwallets
["", "miner", "receiver"]

$ bitcoin-cli -rpcwallet=miner getnewaddress mining
bcrt1qp9nknyu260ulsnpwu7x6a8y55zpr0a52nk56u5

$ bitcoin-cli -rpcwallet=receiver getnewaddress classmate
bcrt1q6mjvw7anvu2a885cstjrmk9jha4p4e46egl40f

$ bitcoin-cli -rpcwallet=miner getaddressinfo <miner mining2 address>
{
  "address": "bcrt1q0tvlxqh4vkfzwuu9qun9d4txwrf76uj7syyvhy",
  "ismine": true,
  ...
  "labels": ["mining2"]
}

$ bitcoin-cli -rpcwallet=receiver getaddressinfo <receiver classmate2 address>
{
  "address": "bcrt1q8xnsl28ymp70jxzmf7gnxx59aaa4tk6nl0cxs2",
  "ismine": true,
  ...
  "labels": ["classmate2"]
}

$ bitcoin-cli -rpcwallet=receiver getaddressinfo <miner mining2 address>   # wrong wallet
{
  "address": "bcrt1q0tvlxqh4vkfzwuu9qun9d4txwrf76uj7syyvhy",
  "ismine": false,
  "solvable": false,
  "labels": []
}
```

Both generated addresses use the `bcrt1...` regtest prefix. Ownership checks confirm each address
belongs only to the wallet that generated it — the `miner` wallet reports `ismine: true` for its
own address and the `receiver` wallet reports `ismine: false` for that same address.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab02`; no separate screenshots were taken for this lab.

## Explanation

Bitcoin Core can hold multiple wallets loaded on a single node at once (here, an empty default
wallet plus `miner` and `receiver`). RPCs like `getnewaddress`, `getaddressinfo`, and `getbalance`
are **wallet-scoped** — they operate on keys, addresses, and balances that live inside one specific
wallet's own storage, not the node as a whole. Because of that, Bitcoin Core needs to know *which*
wallet a call is for, which is exactly what the `-rpcwallet=<name>` flag (or, in code, passing
`Some(wallet_name)` into `RpcClient::call`) supplies.

Calling a wallet-scoped RPC with no wallet context, or with the wrong wallet loaded, doesn't
silently guess — it either fails outright (as seen when I ran a command against a wallet name that
didn't exist, error `-18: Requested wallet does not exist or is not loaded`) or, if the wallet does
exist but simply doesn't control the address in question, returns a "not mine" answer instead of an
error (`"ismine": false` when checking the miner's address from inside the receiver wallet). Both
behaviors matter: mixing up wallet context could make you think you're checking one wallet's funds
or addresses when you're actually looking at (or worse, about to spend from) a completely different
one.
