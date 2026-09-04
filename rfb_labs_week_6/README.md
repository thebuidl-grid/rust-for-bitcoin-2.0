# Week 6 — a regtest Bitcoin wallet in Rust

A CLI wallet built on `rust-bitcoin`, `bdk_wallet`/`bdk_bitcoind_rpc` (BDK), and `bitcoincore-rpc`.
It generates/imports keys from a BIP39 mnemonic, derives a BIP84 (native SegWit) wallet with
separate receive/change keychains, tracks UTXOs and balance by syncing block-by-block against a
Bitcoin Core node, persists all of that to a local SQLite file, and can build, sign, and broadcast
real transactions on regtest.

(`Readme.md` in this directory is the original assignment brief from the instructor and is left
untouched; this `README.md` is the submission's own documentation.)

## How to run it

**Prerequisites:** a Bitcoin Core node running on **regtest** that you can reach over RPC. This
was built and tested against a [Polar](https://lightningpolar.com/) regtest `bitcoind` node
(`polaruser` / `polarpass` on `127.0.0.1:18443`), which is what the defaults below assume — swap
in your own node's `RPC_URL`/`RPC_USER`/`RPC_PASS` if you're using something else (e.g.
`bitcoind -regtest -rpcuser=... -rpcpassword=...` started directly).

```bash
cd rfb_labs_week_6

# Optional: copy .env.example -> .env and edit RPC_URL/RPC_USER/RPC_PASS if not using Polar's
# defaults. WALLET_MNEMONIC can be left unset -- see "Keys and config" below.
cp .env.example .env

# See every subcommand:
cargo run -- --help

# First run: creates the wallet (and, if WALLET_MNEMONIC wasn't set, generates and saves one).
cargo run -- info

# Fund it: mines 101 regtest blocks paying their coinbase reward to a fresh wallet address
# (101 clears the first block's coinbase maturity, so you actually have spendable funds).
cargo run -- fund 101
cargo run -- balance

# Get an address, send to it, then mine a block to confirm the spend:
cargo run -- new-address
cargo run -- send bcrt1q... --amount-sats 1000000 --fee-rate 2
cargo run -- fund 1
cargo run -- balance
cargo run -- transactions
cargo run -- utxos
```

Every command re-syncs with the node first, so `balance`/`transactions`/`utxos` always reflect
current chain state. Wallet state (descriptors, keychain indices, UTXOs, tx history, chain sync
position) is persisted to `wallet.sqlite` after every mutation, so stopping and restarting the
process picks up exactly where it left off — nothing is re-derived or re-synced from scratch.

## Keys and config (no hardcoded secrets)

`src/config.rs` loads everything from the environment / a local `.env` file — nothing sensitive
is in source. If `WALLET_MNEMONIC` isn't set, a fresh 12-word BIP39 test mnemonic is generated on
first run and appended to `.env` (git-ignored) so the same wallet is reused on every subsequent
run. This is disposable regtest/testnet-only test data; the app explicitly refuses to run against
`BITCOIN_NETWORK=bitcoin` (see the check in `Config::load`), so there's no path to accidentally
pointing this at mainnet with real funds.

## Project structure and descriptor choice

```
src/config.rs  -- env/.env loading, mnemonic generation, network guard
src/node.rs    -- bitcoincore-rpc client + connectivity check
src/wallet.rs  -- wallet create/load (SQLite-persisted) + chain sync
src/main.rs    -- clap CLI, dispatches to the above
```

The wallet uses **BIP84** (`wpkh(...)`, native SegWit) descriptors for both keychains, derived
from one BIP32 master key via `bdk_wallet::descriptor::template::Bip84`:

```rust
let external = Bip84(xpriv, KeychainKind::External);  // wpkh(key/84'/1'/0'/0/*) -- receive
let internal = Bip84(xpriv, KeychainKind::Internal);   // wpkh(key/84'/1'/0'/1/*) -- change
```

BIP84 was chosen over legacy P2PKH or wrapped SegWit (BIP49) because native SegWit is the
simplest fully-modern single-key format still universally supported by regtest/testnet tooling
(unlike Taproot/BIP86, which some older RPC helpers don't expect) — see "Stretch goal" below for
where a Taproot (BIP86) descriptor would differ. Using **one master key with two separate BIP84
keychains** (rather than one keychain for everything) keeps receive addresses and change addresses
in separate derivation branches, which is the whole point of "external/internal keychains" in the
assignment brief — it lets a watch-only observer (or just good hygiene) distinguish "addresses I
handed out" from "change I generated for myself."

## Which library did what, and why

- **`bdk_wallet`** owns descriptor management, keychains, address derivation, UTXO/balance
  tracking, transaction building (`TxBuilder`), signing, and SQLite persistence
  (`bdk_wallet::rusqlite`). This is the layer that turns "a BIP32 key + a descriptor template"
  into "a wallet that knows its own UTXOs and can build a spend" — reimplementing coin selection,
  change-output creation, and PSBT construction by hand on top of raw `rust-bitcoin` would just be
  re-deriving BDK's own logic, badly.
- **`bdk_bitcoind_rpc`** bridges BDK's wallet state to a Bitcoin Core node: its `Emitter` walks
  blocks from the node one at a time and reports mempool transactions/evictions, in a form
  `Wallet::apply_block_connected_to`/`apply_unconfirmed_txs` can consume directly (see
  `src/wallet.rs::sync`). This is the "sync wallet state via RPC" requirement.
- **`bitcoincore-rpc`** is used directly (not just through `bdk_bitcoind_rpc`) for everything that
  isn't itself a chain-sync concern: checking the node is reachable (`node.rs`), broadcasting a
  finalized transaction (`send_raw_transaction`), and — for the regtest `fund` helper —
  `generate_to_address`. This is the "connect to a node to broadcast transactions" requirement,
  and it's a deliberately separate concern from syncing: syncing reads chain state into the
  wallet, broadcasting pushes a wallet-built transaction back out.
- **`rust-bitcoin`** types (`Address`, `Amount`, `FeeRate`, `Transaction`, ...) are used directly
  wherever the CLI talks about amounts/addresses/transactions, since `bdk_wallet` re-exports and
  builds on exactly these types rather than wrapping them in its own parallel type system.
- **`clap`** provides the CLI subcommands (stretch goal, see below). **`anyhow`** provides
  `.context(...)`-annotated errors everywhere fallible calls happen, so a bad RPC connection, a
  malformed address, or an invalid fee rate produces a specific message pointing at what went
  wrong instead of a panic. **`dotenvy`** loads `.env` for config.

## Stretch goals attempted

- **A simple CLI** (`src/main.rs`): `info`, `sync`, `balance`, `new-address`, `transactions`,
  `utxos`, `fund <blocks>` (regtest helper), and `send <address> --amount-sats N [--fee-rate R]`
  — a user can check balance, get a new address, and send funds without touching source at all.
- **Basic error handling**: every fallible operation (RPC calls, address/fee-rate parsing, tx
  building/signing/broadcasting) is wrapped with `anyhow::Context` so failures produce a specific,
  readable message and a non-zero exit code rather than a panic. The `Send` path also checks that
  `wallet.sign(...)` actually fully finalized the transaction before broadcasting anything.
- **Raw `rust-bitcoin` instead of BDK — with a working example.** After building/signing a
  transaction but before broadcasting it, `Command::Send` calls `audit_transaction` in
  `src/main.rs`, which uses **only** `bitcoin::Transaction`/`bitcoin::Address` — no `bdk_wallet`
  API at all — to independently recompute the transaction's weight/vsize and decode every output's
  address straight from its raw scriptPubKey bytes:

  ```rust
  fn audit_transaction(tx: &Transaction, network: Network) {
      println!("weight={} vsize={} vbytes", tx.weight(), tx.vsize());
      for (index, output) in tx.output.iter().enumerate() {
          let address = Address::from_script(&output.script_pubkey, network)?;
          println!("output[{index}] {} -> {address}", output.value);
      }
  }
  ```

  The scenario this models is real: `TxBuilder::finish()` and `psbt.extract_tx()` hand you back a
  plain `bitcoin::Transaction`, and at that point you have every reason *not* to just trust that
  BDK's own bookkeeping (fee policy, change detection, coin selection) produced what you actually
  intended — especially right before an irreversible broadcast. Recomputing size/weight and
  decoding each output's destination from raw bytes is a check that's fully independent of BDK's
  internals, which is exactly when reaching for the underlying primitives directly (rather than a
  higher-level wallet API) earns its keep: you're verifying the library's output, not trusting it
  by construction. Sample output from an actual regtest run:

  ```
  Raw rust-bitcoin audit: txid=7cfbfec3... weight=561 vsize=141 vbytes
    output[0] 0.00749578 BTC -> bcrt1qewh87xyevxy7qq4ajua4fsgxa905l5qcmfdspn
    output[1] 0.00250000 BTC -> bcrt1q963dwtcsw2hzps8344vh42n3e2yj253w372803
  ```

## Known limitations / what I'd improve with more time

- **Coin selection is BDK's default**, not handled explicitly (the "explicit coin selection"
  stretch goal wasn't attempted). `bdk_wallet::TxBuilder` uses its built-in
  branch-and-bound/largest-first selection; a `coin_selection(...)` implementation of BDK's
  `CoinSelectionAlgorithm` trait would be the next step to make selection policy explicit.
- **Only BIP84/wpkh is implemented**, not a wpkh-vs-Taproot comparison. Adding a parallel
  `Bip86<Xpriv>` wallet (`tr(key/86'/1'/0'/{0,1}/*)`) alongside the BIP84 one, and comparing e.g.
  transaction vsize for an equivalent spend via the same `audit_transaction` helper, would be a
  natural extension — the descriptor-template machinery in `src/wallet.rs` already generalizes to
  this (swap `Bip84` for `Bip86`, and the key must satisfy `DerivableKey<Tap>` rather than
  `DerivableKey<Segwitv0>`, which an `Xpriv` satisfies either way).
- **No fee bumping / RBF helper.** If a `send` broadcast never confirms, there's currently no
  `bump-fee` subcommand; `bdk_wallet::TxBuilder` supports fee bumping via `build_fee_bump`, which
  would be a small addition on top of the existing `Send` code path.
- **Single-node, single-network assumption.** The RPC client and wallet are both wired to one
  configured `Network`/node; there's no multi-network config profile or Electrum/Esplora fallback
  if a Bitcoin Core node isn't available (BDK supports both as alternate chain sources, but this
  submission deliberately used `bitcoincore-rpc` throughout per the assignment's emphasis on it).

## Proof of a working transaction

See the "How to run it" section above for the exact command sequence; a real regtest run produced:

- A funded wallet after `fund 101`: `confirmed: 100 BTC` (2 of 101 mined blocks past coinbase
  maturity), `immature: 4950 BTC` — correct per BIP34 coinbase maturity (100 confirmations).
- A broadcast spend: `Broadcast transaction: a423ed10e056b1ffc18ad8ed9c1761c237174d2a6904732059cf663b6ae972ab`,
  which appeared as `[unconfirmed]` under `transactions` immediately, then `[confirmed]` after
  `fund 1` mined one more block, with the wallet's `confirmed` balance increasing to match.
