# Rust for Bitcoin 2.0 — Week 6: A Regtest Bitcoin Wallet

**Author:** Emmanuel Okanandu

A small but complete Bitcoin wallet, built on `rust-bitcoin`, BDK (`bdk_wallet`) and
`bitcoincore-rpc`. It generates its own key, derives addresses from a descriptor on separate
external/internal keychains, tracks UTXOs and balance, persists all of that to SQLite, and can
build, sign and broadcast a real transaction against a Bitcoin Core node.

The original assignment brief is preserved at [`ASSIGNMENT.md`](ASSIGNMENT.md) (pulled unmodified
from the upstream `rfb_labs_week_6` scaffold — this file is my submission write-up).

## How to run it

### 1. Prerequisites

- Rust (stable; this was built and tested with `rustc 1.93.1`).
- A `bitcoind` node reachable over RPC, running on **regtest** (or testnet/signet — see
  [Constraints](#constraints)). Any Bitcoin Core build works; nothing wallet-specific is required
  on the node side, since this wallet never uses bitcoind's own wallet.

  ```bash
  bitcoind -regtest -daemon
  # cookie auth is picked up automatically from the default datadir:
  # ~/.bitcoin/regtest/.cookie
  ```

  If your node's cookie file, RPC port, or datadir differ from the defaults, pass them explicitly
  (see `--help` below) or set them in `.env`.

### 2. Build

```bash
cd rfb_labs_week_6
cargo build
```

### 3. Create a wallet

```bash
cargo run -- --rpc-cookie ~/.bitcoin/regtest/.cookie init
```

This generates a fresh BIP39 mnemonic, derives a BIP84 (`wpkh`, native SegWit) master key from
it, builds the external (`.../0/*`) and internal/change (`.../1/*`) descriptors, creates an empty
wallet database (`wallet.sqlite`), and writes everything to `.env` — **which is `.gitignored` and
must never be committed.** The mnemonic is printed once, for backup; the program itself only ever
reads the descriptors back out of `.env`.

Every later command reads `RPC_URL` / `RPC_COOKIE` / `DESCRIPTOR` / etc. from `.env`
automatically (or from the equivalent `--flag`), so once `.env` exists you can drop the
`--rpc-cookie` argument.

### 4. Fund it (regtest only)

```bash
cargo run -- mine --blocks 101
```

Mines 101 regtest blocks directly to the wallet's own next address (101, because a coinbase
output needs 100 confirmations to mature). No bitcoind wallet is needed for this — see
[Architecture](#architecture--descriptor-structure) for why.

### 5. Everyday commands

```bash
cargo run -- balance                                    # sync, then print confirmed/pending/immature/total
cargo run -- utxos                                       # sync, then list tracked UTXOs
cargo run -- address                                      # reveal the next receiving address
cargo run -- change-address                               # reveal the next change address
cargo run -- send --to <address> --amount <sats> [--fee-rate <sat/vB>]   # build, sign, broadcast
```

Run `cargo run -- --help` (or `cargo run -- <command> --help`) for the full flag reference,
including how to point at a testnet/signet node instead of regtest.

## Architecture & descriptor structure

```
src/
├── main.rs          CLI dispatch — one small function per subcommand
├── config.rs         clap Args/Config, env-var backed (mirrors BDK's own bitcoind_rpc example)
├── keys.rs            mnemonic → master xprv → wpkh()/tr() descriptor strings
├── wallet_store.rs    init (create) / open (load) the BDK wallet + its SQLite persister
└── chain.rs            bitcoind RPC client + block/mempool sync via bdk_bitcoind_rpc::Emitter
```

**Why a descriptor, and why this shape.** `init` derives one master extended private key
(`Xpriv`) from a fresh BIP39 seed, then writes it into two descriptors:

```
wpkh(<xprv>/84'/1'/0'/0/*)   external / receiving keychain
wpkh(<xprv>/84'/1'/0'/1/*)   internal / change keychain
```

(`--kind tr` builds the Taproot equivalent at `86'` instead of `84'` — see
[Stretch goals](#stretch-goals-attempted).) This is exactly the descriptor BDK's own
`template::Bip84` builds internally; I wrote it out as a plain string instead of using the
template type because a **string** is what gets persisted to `.env` and handed back to
`Wallet::load()` on every subsequent run — there's no dependency on template types at load time,
and the descriptor is portable enough to import into another BDK- or Core-based wallet as-is.
Separating `0/*` (external) from `1/*` (internal) is BIP44's standard convention and is what lets
BDK tell "money I'm receiving" apart from "my own change" when computing `trusted_pending` vs.
`untrusted_pending` balance — see `Balance` in [`bdk_chain`](https://docs.rs/bdk_chain).

**Why `mine` doesn't need a bitcoind wallet.** `bdk_bitcoind_rpc::Emitter` (see `chain.rs`)
walks blocks and the mempool over plain node RPCs (`getblock`, `getrawmempool`, ...), and works
against a wallet-disabled node. So instead of asking bitcoind for an address to mine to, `mine`
reveals the BDK wallet's *own* next address and mines straight to it — the whole assignment stays
self-contained in this one wallet, with bitcoind used purely as a chain-data/broadcast backend.

**Persistence.** `wallet.persist(&mut db)` is called after every state-changing operation (a
revealed address, an applied block, a broadcast tx), so a `cargo run` process can exit and the
next invocation picks up exactly where it left off — proven in [evidence](#evidence-a-real-regtest-transaction)
below, where every command below `init` is a separate process re-opening `wallet.sqlite`.

### Constraints

- **Network:** the `mine` command refuses to run on anything but regtest; `send`/`sync`/etc. work
  on whichever network `BITCOIN_NETWORK` is set to (regtest, testnet, signet or, deliberately
  untested here, mainnet).
- **No secrets on disk in git:** `init` writes the mnemonic and both private descriptors to
  `.env`, and the UTXO set/history to `wallet.sqlite` — both are `.gitignore`d by this crate. Only
  `.env.example` (no real values) is committed.

## Which libraries, and why

| Crate | Used for | Why this one |
|---|---|---|
| [`bdk_wallet`](https://docs.rs/bdk_wallet) | Descriptor parsing, keychain-aware address derivation, UTXO/balance tracking, PSBT tx building, signing, SQLite persistence | This is the actual hard part of a wallet — correct gap-limit address derivation, coin selection, and change handling are exactly what BDK exists to get right instead of reimplementing |
| [`bdk_bitcoind_rpc`](https://docs.rs/bdk_bitcoind_rpc) | Turns bitcoind's blocks/mempool into updates `bdk_wallet::Wallet` can apply, and broadcasts | It's BDK's own bridge to `bitcoincore-rpc` (re-exported as `bdk_bitcoind_rpc::bitcoincore_rpc`), so the RPC types it hands back are guaranteed to match what `Emitter` expects — no second, independently-versioned `bitcoincore-rpc` dependency to keep in sync |
| `rust-bitcoin` (via `bdk_wallet::bitcoin`) | `Address` parsing/validation, `Amount`, `FeeRate`, `Xpriv` | See the raw-`rust-bitcoin` example right below — some things are plain Bitcoin data types, not wallet operations |
| `bip39` | Mnemonic generation and seed derivation (same crate as Week 5) | Standard, minimal, no wallet opinions baked in |
| `clap` (`derive`, `env`) | CLI parsing, with every flag backed by an env var | Matches the pattern in BDK's own `bitcoind_rpc` example, so `.env` written by `init` is self-documenting |
| `dotenvy` | Loads `.env` before `clap` reads `std::env` | So a wallet created by `init` "just works" on every later invocation from the same directory |
| `anyhow` | Error propagation in a binary crate | No public API to design error types around — `anyhow::Result` everywhere is the right amount of ceremony |

### A scenario where raw `rust-bitcoin` was the right tool, not BDK

`cmd_send` in `main.rs` validates the caller-supplied destination address like this:

```rust
let to_address = Address::from_str(to)
    .with_context(|| format!("'{to}' is not a valid Bitcoin address"))?
    .require_network(cfg.network)
    .with_context(|| format!("'{to}' is not a valid address for {}", cfg.network))?;
```

This is plain `rust-bitcoin::Address`, not anything from BDK. A *destination* address for a send
is, by definition, not one of this wallet's own tracked scriptPubKeys — there's nothing for BDK's
keychain/descriptor machinery to do with it. It just needs parsing and a network check, which is
exactly what `rust-bitcoin`'s `Address` type is for. Reaching for a BDK API here would mean
inventing one, since BDK's address-related methods (`reveal_next_address`, `peek_address`, ...)
are all about the wallet's *own* addresses, not arbitrary external ones.

## Stretch goals attempted

- **Both `wpkh` and `tr` descriptors** (`init --kind wpkh` / `init --kind tr`, BIP84 vs BIP86). I
  ran both end-to-end on a fresh regtest wallet each — mined coinbase to a `bcrt1q...` (SegWit
  v0) address and to a `bcrt1p...` (Taproot) address, and successfully signed and broadcast a
  spend from each, confirming both the ECDSA and BIP340 Schnorr signing paths work through the
  same `wallet.sign()` call.
- **A CLI**: `init` / `address` / `change-address` / `sync` / `balance` / `utxos` / `send` /
  `mine`, all through `clap` — no code editing needed to check a balance, get an address, or send
  funds.
- **Basic error handling**: every RPC/IO/signing failure returns a `Result` with a specific,
  human-readable `anyhow::Context` message (e.g. *"no DESCRIPTOR configured — run `init` first"*,
  *"'not-an-address' is not a valid Bitcoin address"*) instead of panicking; `init` refuses to
  silently overwrite an existing `.env`/database unless `--force` is passed.
- The raw-`rust-bitcoin`-vs-BDK example above.

Coin selection was **not** overridden — it's listed under known limitations below.

## Evidence: a real regtest transaction

Below is an unedited transcript from a real run against a local regtest `bitcoind`, in the order
it happened. Every line after `init` is a **separate `cargo run` process**, re-opening
`wallet.sqlite` and `.env` from disk each time — this is also the persistence proof required by
the grading rubric, since nothing here is kept alive in memory between commands.

```
$ cargo run -- --rpc-cookie ~/.bitcoin/regtest/.cookie init
Wallet initialized on regtest.

Recovery phrase (write this down, it is only shown once):
  <redacted for this README — even a disposable regtest mnemonic shouldn't end up in git history>
First receiving address: bcrt1qrs0zdk2r2dxndm378jqcvzf490t20ypv0yq9vm
Descriptors and the phrase above were written to .env — do not commit that file.

$ cargo run -- mine --blocks 101
Mined 101 block(s) to bcrt1q9xrshmfp7vfv6qs0ped5vmqwydq2kcsnm7vkxh.
Balance after sync: 5050 BTC

$ cargo run -- balance
confirmed:         100 BTC
trusted pending:   0 BTC
untrusted pending: 0 BTC
immature:          4950 BTC
total:             5050 BTC

$ cargo run -- address
bcrt1qwzlnqccf45nelz9n9wc60sk4dgkyg83pfmh99w

$ cargo run -- send --to bcrt1qwzlnqccf45nelz9n9wc60sk4dgkyg83pfmh99w --amount 1234567 --fee-rate 2
Broadcast txid: 515e412e75e5ee0fb37e11e48bf3a4d27f4de00ddad167c332ecbd69736aaeeb
New balance (unconfirmed until mined): 5050 BTC

$ cargo run -- mine --blocks 1
Mined 1 block(s) to bcrt1qea70cygp2ueyx8hr9eav5rnw5p9a20fe63z4a2.
Balance after sync: 5100 BTC

$ cargo run -- utxos
...
515e412e75e5ee0fb37e11e48bf3a4d27f4de00ddad167c332ecbd69736aaeeb:0  0.01234567 BTC  External  derivation index 2
515e412e75e5ee0fb37e11e48bf3a4d27f4de00ddad167c332ecbd69736aaeeb:1  49.98765152 BTC  Internal  derivation index 0
```

Independently verified against the node itself, outside this wallet entirely:

```
$ bitcoin-cli -regtest getrawtransaction 515e412e...36aaeeb true
{
  "txid": "515e412e75e5ee0fb37e11e48bf3a4d27f4de00ddad167c332ecbd69736aaeeb",
  ...
  "vin": [{ "txid": "6a6c15eb...deff13", "vout": 0, "txinwitness": [ "3044...01", "0320a3..." ] }],
  "vout": [{ "value": 0.01234567, "scriptPubKey": {
    "desc": "addr(bcrt1qwzlnqccf45nelz9n9wc60sk4dgkyg83pfmh99w)#8zve7pnq" } }]
}
```

The `0.01234567` output and destination address match the `send` command exactly, and the
`Internal`-keychain UTXO at index 0 confirms the change output landed on the change keychain, not
mixed into the receiving one.

## Known limitations / what I'd improve with more time

- **Coin selection is BDK's default** (`DefaultCoinSelectionAlgorithm`, largest-first-ish with
  BnB fallback) — I didn't override it, so `send` can't be told to prefer specific UTXOs or avoid
  particular ones. Exposing `TxBuilder::coin_selection`/`::add_utxos` behind a `--utxo` flag would
  be the natural next step.
- **Single-signature only.** The descriptors are single-key `wpkh`/`tr`; no multisig support.
- **No RBF/CPFP helpers.** `send` broadcasts once; there's no `bump-fee` command if a transaction
  gets stuck (this is where BDK's `TxBuilder::bump_fee` would come in).
- **No fee estimation.** `--fee-rate` defaults to a flat 1 sat/vB and must be set explicitly for
  anything else; wiring up `estimatesmartfee` via `bitcoincore-rpc` would remove the guesswork.
- **`sync` walks from `--start-height 0` by default.** Fine on a short-lived regtest chain; on a
  long-lived testnet node you'd want to persist and reuse the wallet's actual birthday height to
  avoid re-scanning from genesis every time.
- **One wallet per directory.** `.env` and `wallet.sqlite` are both relative to the current
  directory, so running two wallets means two directories — there's no `--wallet-name` profile
  concept.
