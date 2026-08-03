# Lab 02 — Wallets and addresses

## Commands used

Rust: `cargo run --example run` (calls the following in sequence):
- `createwallet miner` / `createwallet receiver` / `createwallet alice` (or `loadwallet` if already created from a prior run) — via `ensure_wallet`
- `listwallets` — via `list_wallets`
- `getnewaddress "mining"` (wallet: miner) — via `get_new_address`
- `getnewaddress "classmate"` (wallet: receiver) — via `get_new_address`
- `getaddressinfo <address>` (wallet: miner, then receiver) — via `address_belongs_to_wallet`, checking the `ismine` field

## Terminal output

=== Lab 02: wallets ===
wallets: ["", "miner", "receiver", "alice"]
miner address: bcrt1qmy6upf6793ax80sjque9tx2hjwsxwe3phcf50f
receiver address: bcrt1qd0jasulp7k5xx3yrrg4s2jelnu526dj47qmskq
miner owns its address: true
receiver owns its address: true

## Evidence references

Screenshot: `evidence/lab02.png`

## Explanation

Bitcoin Core supports multiple independent wallets on a single node. The empty string `""` in the `listwallets` output is the node's default wallet that existed before I created any of my own — `miner`, `receiver`, and `alice` are the three separate wallets I created for this lab.

Each wallet is a completely separate keystore with its own set of addresses, its own balance, and its own transaction history — Bitcoin Core doesn't merge them. Because of that, every RPC call that touches a specific wallet's data (creating an address, checking a balance, sending a payment) needs to know *which* wallet it's operating on. That's the purpose of `-rpcwallet=<name>` (or, in my Rust code, passing `Some(wallet_name)` into `client.call`) — it tells the node "run this RPC against this specific wallet," rather than the node's default. Calls that don't need wallet context, like `getblockchaininfo`, don't take that parameter at all.

An address is not the same thing as a wallet — a wallet can (and usually does) generate many addresses. `getaddressinfo` combined with the `ismine` field is how I confirmed each address genuinely belongs to the wallet that generated it, rather than just assuming it does — both came back `true`, correctly linking `miner`'s address to the `miner` wallet and `receiver`'s address to the `receiver` wallet.