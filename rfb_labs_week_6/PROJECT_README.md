# Week 6 — Bitcoin Wallet in Rust

A minimal, descriptor-based Bitcoin wallet for **regtest / testnet** built with
`bdk_wallet`, `bdk_bitcoind_rpc`, and `bitcoincore-rpc`.

---

## Features

| Requirement | Implementation |
|---|---|
| Key generation / import | BIP-39 mnemonic → BIP-84 wpkh descriptors |
| External + internal keychains | `KeychainKind::External` / `KeychainKind::Internal` |
| UTXO tracking & balance | `bdk_wallet::Wallet::balance()` after sync |
| Persistent state | SQLite via `bdk_wallet`'s built-in `rusqlite` feature |
| Construct, sign, broadcast | `build_tx` → PSBT → `sign` → `sendrawtransaction` |
| Node connection | `bdk_bitcoind_rpc::Emitter` + `bitcoincore-rpc` |

---

## Setup

### Prerequisites

- Rust 1.85+ (`rustup default stable`)
- A running Bitcoin Core node in **regtest** mode

Start a regtest node:

```bash
bitcoind -regtest -daemon \
  -rpcuser=bitcoin -rpcpassword=bitcoin \
  -rpcport=18443 -server
```

Mine some initial blocks so the coinbase matures:

```bash
bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin \
  generatetoaddress 101 $(bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin getnewaddress)
```

### Configuration

Copy `.env.example` to `.env` and fill in your RPC credentials:

```bash
cp .env.example .env
# Edit .env — set RPC_USER, RPC_PASS, and optionally MNEMONIC
```

**Never commit `.env` or paste a real mnemonic into any source file.**

### Build

```bash
cargo build --release
```

---

## Usage

All commands load configuration from `.env` (or shell environment variables).

### Get a receiving address

```bash
cargo run -- address
```

On first run a fresh 12-word mnemonic is printed — copy it to `MNEMONIC` in
`.env` to make the wallet persistent across `cargo clean` runs.

### Sync against the node

```bash
cargo run -- sync
```

Walks every new block from the wallet's current tip and ingests the mempool.
State is persisted to `wallet.sqlite` after each block.

### Check balance

```bash
cargo run -- balance
```

### Send funds

```bash
cargo run -- send --to <ADDRESS> --amount-sats <SATOSHIS>
```

The wallet syncs first, then constructs, signs, and broadcasts a transaction.
RBF is signalled (`nSequence = 0xFFFFFFFD`) on all inputs.

Example (regtest):

```bash
DEST=$(bitcoin-cli -regtest -rpcuser=bitcoin -rpcpassword=bitcoin getnewaddress "" bech32)
cargo run -- send --to "$DEST" --amount-sats 100000
```

---

## Architecture & library choices

### `bdk_wallet` (descriptor engine)

Used for everything wallet-internal: key derivation, address generation, coin
selection, PSBT construction, signing, and UTXO indexing.  BDK's
descriptor-first model means the wallet logic is completely agnostic of the
underlying key type — swapping `wpkh` for `tr` (Taproot) requires only
changing two descriptor strings.

The **SQLite persistence backend** (`features = ["rusqlite"]`) is built into
`bdk_wallet`: a single `wallet.persist(&mut conn)` call atomically writes any
`ChangeSet` produced since the last persist.  This satisfies the "wallet
survives a restart" requirement without any custom serialisation code.

### `bdk_bitcoind_rpc` (chain source)

Used to sync block-by-block via `Emitter::next_block`.  This crate purposely
avoids Bitcoin Core's own wallet RPC, so the approach works even with a
wallet-disabled node.  The `Emitter` is given the wallet's current `CheckPoint`
and only fetches blocks newer than that tip, making re-syncs after a restart
very cheap.

### `bitcoincore-rpc` (broadcast + fee estimation)

Used directly for two operations that `bdk_bitcoind_rpc` does not expose:
- `estimate_smart_fee` — to pick a realistic feerate before building a PSBT
- `send_raw_transaction` — to broadcast the final signed transaction

This is the "raw `rust-bitcoin` / direct RPC" layer: BDK hands us a fully
signed `bitcoin::Transaction`; we serialise it with `consensus::serialize` and
push it to Core.

### `clap` (CLI)

A simple four-command interface (`address`, `balance`, `sync`, `send`) so the
wallet can be used without editing source code.

### `dotenv` + env vars (secrets)

All credentials and the optional mnemonic come from environment variables,
loaded from `.env` at startup.  No private material ever appears in source
files or `Cargo.toml`.

---

## Known limitations / future improvements

- **Testnet**: the code targets regtest; switching to testnet/signet requires
  changing `Network::Regtest` → `Network::Testnet` and updating RPC port/auth.
- **Fee estimation on regtest**: `estimate_smart_fee` always returns `None` on a
  fresh regtest chain (not enough blocks).  The wallet falls back to
  `FeeRate::BROADCAST_MIN` in that case.
- **No gap-limit scan on import**: importing an existing mnemonic after the
  wallet DB is gone will only reveal addresses from index 0 onward; a full
  rescanning loop over the gap limit would be needed for production use.
- **No passphrase support**: BIP-39 passphrase support is wired as `None`;
  trivial to expose via a `MNEMONIC_PASSPHRASE` env var.
- **Single account**: only the first BIP-44/84 account (`0'`) is used.
