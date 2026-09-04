# Week 6 — Bitcoin Wallet in Rust

A CLI wallet that runs against a local regtest `bitcoind`, built on `bdk_wallet` for
descriptor/keychain/UTXO management and `bdk_bitcoind_rpc` for talking to the node.

## Setup

You need `bitcoind`/`bitcoin-cli` on your PATH (Bitcoin Core 25+). No Docker or Polar
required — this runs a plain regtest node directly.

```bash
# start a regtest node (adjust paths as you like)
mkdir -p /tmp/rfb_week6_regtest
cat > /tmp/rfb_week6_bitcoin.conf << 'EOF'
regtest=1
server=1
txindex=1
fallbackfee=0.0001
[regtest]
rpcuser=rfbuser
rpcpassword=rfbpass
rpcport=18443
EOF
bitcoind -datadir=/tmp/rfb_week6_regtest -conf=/tmp/rfb_week6_bitcoin.conf -daemon
```

Then set up the wallet's own config:

```bash
cp .env.example .env   # edit if you changed any of the values above
cargo run -- init      # generates a mnemonic if MNEMONIC isn't already set,
                        # prints it once, and creates the SQLite wallet DB
```

`init` prints a `MNEMONIC="..."` line — paste it into `.env` so later commands can find
it. Nothing is ever written to disk by this tool except the SQLite wallet database
(`BDK_DB_PATH`, which holds no private key material — descriptors with keys live only
in `.env`).

## Commands

```bash
cargo run -- init                          # create the wallet, print first address
cargo run -- address                       # reveal next receiving (external) address
cargo run -- change-address                # reveal next change (internal) address
cargo run -- sync                          # pull new blocks/mempool state, print balance
cargo run -- utxos                         # sync, then list UTXOs
cargo run -- send <address> <sats> [--fee-rate N]   # sync, build, sign, broadcast
cargo run -- mine [count]                  # regtest-only: mine blocks to a wallet address
```

Regtest coinbase outputs need 100 confirmations to mature, so a fresh wallet needs:

```bash
cargo run -- mine 101
cargo run -- sync
```

before `balance.confirmed` shows anything spendable.

## Project structure

```
src/
  main.rs    CLI (clap) — dispatches to the modules below
  config.rs  reads .env / real env vars into a Config struct
  keys.rs    mnemonic -> BIP84 wpkh descriptors (external + internal)
  node.rs    builds the bitcoind RPC client from Config
  wallet.rs  wallet open/create + sync (Emitter-driven block/mempool ingestion)
```

Each command is a fresh process: it opens the SQLite-backed wallet, syncs against
bitcoind, does the thing, persists, and exits. There's no daemon — every command that
needs current state calls `sync` first rather than trusting stale data on disk.

## Library choices

- **`bdk_wallet`** owns the actual wallet logic: descriptor parsing, keeping the
  external/internal keychains separate, UTXO tracking, coin selection, PSBT
  construction and signing, and SQLite persistence (`rusqlite` feature). This is
  exactly the layer BDK exists to save you from hand-rolling.
- **`bdk_bitcoind_rpc`** bridges bdk_wallet to a real node. `Emitter` walks blocks
  from the wallet's last checkpoint forward and reports reorgs/mempool changes in a
  form `Wallet::apply_block_connected_to` can consume directly — this is what makes
  the wallet "not purely offline."
- **`bitcoincore_rpc`** (re-exported through `bdk_bitcoind_rpc`, so it's not a
  separate dependency) is used directly for `send_raw_transaction` (broadcasting the
  final signed tx) and `generate_to_address` (the `mine` dev command). Both are
  one-shot RPC calls that don't need to go through BDK's chain-source machinery.
- No raw `rust-bitcoin` calls outside what `bdk_wallet` already re-exports —
  `Address`, `Amount`, `FeeRate` all come from `bdk_wallet::bitcoin`. There wasn't a
  case here where raw `rust-bitcoin` bought anything BDK's `TxBuilder`/`Wallet::sign`
  didn't already cover; the whole point of this assignment was to use the ecosystem
  crates, and reaching past BDK would've meant re-implementing what it does well.

## Descriptor choice

BIP84 `wpkh` for both keychains (`m/84h/1h/0h/0/*` external, `m/84h/1h/0h/1/*`
internal). Chosen over taproot because it's the format the Week 5 labs already built
address/script logic around, and native SegWit's discounted witness weight is the
actual reason wallets default to it — taproot buys signature aggregation and
script-privacy properties this simple single-key wallet has no use for yet.

## Known limitations / what I'd improve with more time

- Single hardcoded account (`0'`) and a fixed BIP84 path — no multi-account support.
- `send` always fully drains coin selection to BDK's default algorithm; no explicit
  UTXO selection.
- No taproot descriptor option, so there's no `wpkh` vs `tr` comparison here (stretch
  goal not attempted, given the time available).
- Error handling is `anyhow`-flavored bail/ensure — fine for a CLI, but a library
  consumer of these modules would want typed errors.
- `Mine` mines to a freshly revealed wallet address every time rather than reusing
  the last one — harmless on regtest, just burns through the gap limit faster than
  necessary.

## Proof of a working transaction

Verified live against a local regtest node during development: mined 101 blocks to
mature a coinbase, `sync` reported a spendable balance, `send` built/signed/broadcast
a transaction, and `bitcoin-cli getrawtransaction <txid> true` confirmed it in the
node's mempool before being mined into the next block. Re-running `sync` in a fresh
process afterward showed the same balance, confirming SQLite persistence survives a
restart.
