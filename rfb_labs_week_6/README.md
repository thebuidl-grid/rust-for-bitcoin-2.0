# Week 6 — Regtest Bitcoin Wallet

A minimal Bitcoin wallet built in Rust, running against `regtest`. It generates
keys from a BIP39 mnemonic, derives BIP84 (`wpkh`) receive/change descriptors,
tracks UTXOs and balance, persists its state to SQLite, and can build, sign,
and broadcast transactions through a connection to a local `bitcoind` node.

See [`Readme.md`](./Readme.md) for the original assignment brief. This file
covers setup, design rationale, and proof that it works.

## Setup

### 1. Run a regtest node

You need a local `bitcoind` reachable over RPC. Any of the following works:

```bash
bitcoind -regtest -datadir=/path/to/a/regtest/datadir \
  -server=1 -rpcbind=127.0.0.1 -rpcallowip=127.0.0.1 -fallbackfee=0.0002
```

Cookie authentication (the default `bitcoind` produces) is the simplest path —
point `RPC_COOKIE_FILE` at `<datadir>/regtest/.cookie` (see below).

### 2. Configure `.env`

```bash
cp .env.example .env
```

Then edit `.env`:

| Variable | Meaning |
|---|---|
| `BITCOIN_NETWORK` | `regtest` or `testnet` |
| `RPC_URL` | `host:port` of `bitcoind`'s RPC server, e.g. `127.0.0.1:18443` |
| `RPC_COOKIE_FILE` | path to `bitcoind`'s `.cookie` file (leave `RPC_USER`/`RPC_PASS` empty when using this) |
| `RPC_USER` / `RPC_PASS` | alternative to cookie auth |
| `WALLET_DB_PATH` | where the wallet's SQLite file lives (default `wallet.sqlite`) |
| `MNEMONIC` | leave empty on first run — the wallet generates one and prints it once; copy it back into `.env` to keep using the same wallet |

`.env` is gitignored. Never commit it — see [Constraints](#constraints-followed) below.

### 3. Run it

```bash
cargo build
cargo run -- --help
```

Subcommands:

```bash
cargo run -- demo               # full create → fund → sync → send → confirm walkthrough (regtest only)
cargo run -- balance            # sync, then print balance + UTXO count
cargo run -- address            # reveal & print the next receive address
cargo run -- address --internal # reveal & print the next change address
cargo run -- sync                # sync the wallet's view of the chain
cargo run -- send --to <address> --amount <btc> [--fee-rate <sat/vb>]
cargo run -- fund --blocks <n>   # regtest only: mine blocks to a fresh address
```

`demo` is the same walkthrough used to produce the proof of a working
transaction below — run it against a fresh `.env`/`wallet.sqlite` to
reproduce it.

## Design & library choices

| Library | Used for | Why |
|---|---|---|
| **BDK** (`bdk_wallet`) | Descriptor parsing, external/internal keychain management, address derivation, UTXO/balance tracking, PSBT construction and signing, SQLite persistence (`rusqlite` feature) | Reimplementing BIP32 derivation, a UTXO index, coin selection, and a persistence schema from scratch is multi-week work that BDK already does correctly. Using it let the time go into the parts that actually prove the wallet works (a real broadcast, real persistence) instead of re-deriving infrastructure BDK already ships. |
| **`bdk_bitcoind_rpc`** | `Emitter`-driven block-by-block + mempool sync against `bitcoind` | This crate *is* the wallet's use of `bitcoincore-rpc` for sync — it re-exports and wraps `bitcoincore_rpc::Client` internally rather than being an alternative to it. Using it satisfies "connect to a node via bitcoincore-rpc" for everything that isn't regtest chain administration. |
| **`bitcoincore-rpc`** (direct) | Regtest-only chain control: `generatetoaddress` (funding), `getblockchaininfo` (connectivity check), `sendrawtransaction` (broadcast) | These aren't wallet operations — mining blocks and broadcasting a raw transaction are node/chain administration, so they go through a plain `bitcoincore_rpc::Client` rather than through BDK. |
| **`rust-bitcoin`** (direct) | Post-signing transaction introspection in `tx::send` — `Psbt::extract_tx()`, then `Transaction::compute_txid()`, `.weight()`, `.vsize()`, and `consensus::encode::serialize_hex()` | See [rust-bitcoin vs BDK](#rust-bitcoin-vs-bdk) below. |
| `dotenvy` | Loading `.env` | Keeps secrets (mnemonic, RPC credentials) out of source and out of `Cargo.toml`. |
| `clap` | The CLI (`src/cli.rs`) | Stretch goal: operate the wallet (`balance`/`address`/`send`/`sync`/`fund`) without editing code. |
| `anyhow` / `thiserror` | Error handling | `thiserror` gives typed errors per module (`ConfigError`, `WalletError`, `NodeError`, `TxError`); `anyhow` propagates them cleanly out of `main`. |

### Descriptor structure

The wallet derives two **BIP84** (native segwit, `wpkh`) descriptors from a
single BIP39 mnemonic, at `m/84h/1h/0h/0` (external/receive) and
`m/84h/1h/0h/1` (internal/change) — the standard account-0 split for a
single-sig segwit wallet. `wpkh` was chosen over `tr` (Taproot) as the
primary format because it's the simpler, more universally-supported case for
the minimum requirements; see below for where Taproot was considered.

### rust-bitcoin vs BDK

BDK's `TxBuilder`/PSBT flow handles everything the minimum requirements need
— building, signing, and finalizing a transaction. It doesn't, however,
surface low-level introspection like exact transaction weight, vsize, or a
raw hex dump. In `src/tx.rs`, once BDK has signed and finalized the PSBT:

```rust
let tx = psbt.extract_tx().map_err(|e| TxError::Extract(e.to_string()))?;
println!("tx txid: {}", tx.compute_txid());
println!("tx weight: {}", tx.weight());
println!("tx vsize: {}", tx.vsize());
println!("tx raw hex: {}", serialize_hex(&tx));
```

`psbt.extract_tx()` returns a plain `bitcoin::Transaction` — from that point
on this is raw `rust-bitcoin`, not BDK. This is the kind of thing you'd reach
for `rust-bitcoin` directly for: BDK's `TxBuilder` is the right tool for
*building* a transaction, but once you have a finalized one and want to
inspect its wire format directly (weight accounting, manual broadcast,
custom parsing), going straight to `rust-bitcoin`'s own types is simpler than
looking for (or not finding) an equivalent BDK convenience method.

### Constraints followed

- Regtest only (the code also accepts `testnet` in config, untested here).
- No hardcoded keys: the mnemonic comes from `.env` (gitignored) or is
  generated fresh at runtime and printed once with an explicit warning to
  save it and never commit it. `.env.example` only ever contains placeholders.
  Descriptor strings are only ever printed in their **public** (xpub) form —
  `keys::descriptors_from_mnemonic` returns both a private (tprv, used to
  build a signing wallet) and public (xpub, safe to log) variant of each
  descriptor, specifically so a private key never ends up in stdout.

## Known limitations

- Single-signature only — no multisig, no descriptor types beyond `wpkh`.
- No fee bumping / RBF.
- `sync_wallet` runs synchronously to completion; there's no background sync
  or graceful interrupt (the upstream `bdk_wallet` example this is based on
  uses a threaded channel + `ctrlc` handler for that, which felt like
  unnecessary complexity for a one-shot CLI).
- Coin selection uses BDK's default algorithm — no explicit strategy.
- Only tested against regtest; testnet is wired into `Config` but unverified.
- `bitcoin-cli gettransaction` doesn't work against this setup and isn't used
  for verification below — it's a Bitcoin Core *wallet* RPC, and this project
  deliberately never loads a wallet in Core (all wallet state lives in our
  own SQLite file via BDK). `getrawtransaction <txid> true` is the correct,
  wallet-independent way to inspect a transaction on the node, and is what's
  used below.

## Proof of a working transaction

Produced by running `cargo run -- demo` against a freshly-wiped regtest chain
(a fresh `bitcoind -regtest` datadir, no prior blocks) and a fresh
`wallet.sqlite`/`.env`.

Wallet output:

```
connected to node: chain=regtest blocks=0
mined 101 blocks to bcrt1q3y6pj4658kgtks5wyhft09lf2dp4vsw446p2d0
balance: total=5050 BTC confirmed=100 BTC trusted_pending=0 BTC
utxos: 101
tx txid: 80e07aedcef5e376374e2ed03361d36e3a2391419738294f863cd3a2f4010094
tx weight: 561
tx vsize: 141
broadcast txid: 80e07aedcef5e376374e2ed03361d36e3a2391419738294f863cd3a2f4010094
balance after 1 confirmation: total=5100 BTC confirmed=149.99999719 BTC
```

Independently verified on the node (no wallet loaded in Core — see
[Known limitations](#known-limitations)):

```bash
$ bitcoin-cli -regtest getrawtransaction 80e07aedcef5e376374e2ed03361d36e3a2391419738294f863cd3a2f4010094 true
{
  "txid": "80e07aedcef5e376374e2ed03361d36e3a2391419738294f863cd3a2f4010094",
  "vsize": 141,
  "weight": 561,
  "vout": [
    { "value": 1.00000000, "address": "bcrt1qaxuugq894m0fedg39lpg4wdxt002uvur97yfu9", ... },
    { "value": 48.99999719, "address": "bcrt1qygef5cj28tskq07en2d86ahfw8sc62vcnn246v", ... }
  ],
  "confirmations": 1,
  ...
}
```

`confirmations: 1` confirms the broadcast transaction is real, mined, and
matches exactly what the wallet reported (1 BTC out, ~49 BTC change, same
txid, same weight/vsize) — the wallet's own view of the world agrees with
the node's.

## Bug found & fixed during development

`descriptors_from_mnemonic` originally called `Descriptor::to_string()` on
the derived descriptor to build the wallet — but that method only
serializes the **public** half of a descriptor; the private key lives in a
separate `KeyMap` returned alongside it. That silently produced a
watch-only wallet with no signing capability, which only surfaced as
`signing did not fully finalize the PSBT` once transaction signing was
implemented. Fixed by using `to_string_with_secret(&keymap)` for the
descriptor strings used to build the wallet, while keeping a separate
public-only variant for anything that prints or logs a descriptor.
