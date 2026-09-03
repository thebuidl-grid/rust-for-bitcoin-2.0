# Assignment: Building a Bitcoin Wallet in Rust

## Goal

Build a functioning Bitcoin wallet in Rust (regtest) that demonstrates you can use the libraries covered in class effectively.

## Minimum Requirements

Your wallet must be able to:

1. **Generate or import keys** and derive a wallet from a descriptor.
2. **Generate addresses** from both an external (receiving) and internal (change) keychain.
3. **Track UTXOs and calculate balance** for the wallet.
4. **Persist wallet state locally** (e.g. with SQLite) so the wallet can be closed and reopened without losing track of its own state.
5. **Construct, sign, and broadcast a transaction** on testnet.
6. **Connect to a Bitcoin node** (via `bitcoincore-rpc`) to sync wallet state or broadcast transactions, i.e., your wallet should not be purely offline.

## Stretch Goals

Pick any of these if you want to push further:

- Support multiple descriptor types (e.g. compare `wpkh` vs `tr` Taproot)
- Build a simple CLI so a user can check balance, get a new address, and send funds without editing code
- Handle coin selection explicitly rather than relying on defaults
- Add basic error handling/logging that would make this usable by someone other than you
- Explain (in your README) a scenario where you reached for raw `rust-bitcoin` instead of BDK, and demonstrate it with a small code example

## Acceptance Criteria

1. **A PR TO THE RFB GITHUB** (source code, `Cargo.toml`, etc.) as a zip or a link to a repository.
2. **A README** that includes:
   - How to run your wallet (setup instructions, any node/config required)
   - A short explanation (project or descriptor structure, and why you chose it)
   - Which libraries you used where, and why (e.g. "I used `bitcoincore-rpc` for X, and BDK for Y, because...")
   - Any known limitations or things you'd improve with more time

## Constraints

- Testnet or regtest only.
- Do not hardcode private keys or seed phrases in files you submit, use a `.env`, config file, or generate fresh test keys. (This is also good practice for real-world Bitcoin development.)
- You may use any crates that support your chosen libraries (e.g. `dotenv`, `clap`, `tokio`), but the wallet logic itself should go through `rust-bitcoin`, `bitcoincore-rpc`, and/or BDK.

## Reference Material

- rust-bitcoin docs: https://docs.rs/bitcoin/0.32.102/bitcoin/index.html
- Bitcoin Core RPC reference: https://developer.bitcoin.org/reference/rpc/
- BDK Wallet docs: https://docs.rs/bdk_wallet/latest/bdk_wallet/index.html

---

## Grading Rubric (100 points)

| Category | Criteria |
|---|---|
| **Correctness — Core Functionality** | Wallet generates keys/addresses correctly from a descriptor; external and internal keychains are properly separated |
| **Correctness — UTXO & Balance Tracking** | Wallet accurately tracks UTXOs and reports correct balance after syncing |
| **Correctness — Transactions** | Wallet successfully creates, signs, and broadcasts a transaction on testnet/regtest; txid is verifiable |
| **Persistence** | Wallet state survives a restart (SQLite or equivalent) without needing to re-derive/re-sync from scratch |
| **Node Integration** | Wallet correctly connects to and communicates with a Bitcoin node (RPC or BDK-supported connection) |
| **Architecture & Library Use** | Sensible use of rust-bitcoin / bitcoincore-rpc / BDK together, student can justify *why* each was used where|
| **Code Quality** | Reasonably organized, readable, compiles cleanly, handles at least basic errors (doesn't just panic on bad input) |
| **README & Documentation** | Clear setup instructions, explains design decisions, includes proof of a working transaction |
| **Stretch Goals (bonus)** | Any stretch goal attempted and working |

## Timeline

- **Due:** 5th Septermber 2026

---

# Submission: `rfb-wallet`

A regtest Bitcoin wallet CLI built on `bdk_wallet` (descriptors, UTXO tracking, PSBT
signing, SQLite persistence), `bdk_bitcoind_rpc` + `bitcoincore-rpc` (node sync and
broadcast), and raw `rust-bitcoin` for one scenario BDK can't express (see Stretch
goals). Author: Ikeh Celestine.

## Setup

1. **Start a regtest node.** Either run your own `bitcoind -regtest`, or use the
   bundled `docker-compose.yml`:

   ```bash
   docker compose up -d
   ```

   This starts bitcoind on regtest with RPC on `127.0.0.1:18443`, user `rfbuser`,
   password `rfbpass` (matching `.env.example`).

2. **Configure the wallet.**

   ```bash
   cp .env.example .env
   ```

   The defaults in `.env.example` already match the docker-compose node. Adjust
   `RPC_URL`/`RPC_USER`/`RPC_PASS` (or set `RPC_COOKIE` to a cookie file) if you're
   pointing at a different node.

3. **Initialize the wallet.**

   ```bash
   cargo run -- init
   ```

   This generates a fresh BIP39 mnemonic (saved into `.env` as `MNEMONIC=...` so
   later commands pick it up automatically), derives external/internal descriptors,
   creates `wallet.sqlite`, and prints the first receiving address.

4. **Fund it and sync.** With the docker-compose node, mine to your wallet's address
   (coinbase needs 100 confirmations to mature):

   ```bash
   ADDR=$(cargo run -q -- address)
   docker exec rfb-week6-bitcoind bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
     generatetoaddress 101 "$ADDR"
   cargo run -- balance
   ```

## Commands

| Command | What it does |
|---|---|
| `init` | Generate/load the mnemonic, derive descriptors, create the wallet DB, print the first address |
| `address [--change]` | Reveal the next receive (or change) address |
| `sync` | Sync against bitcoind: replay new blocks, then pull in the mempool |
| `balance` | Sync, then print confirmed/pending/immature/total balance |
| `utxos` | Sync, then list tracked UTXOs |
| `send --to <addr> --amount <sats> [--fee-rate <sat/vb>]` | Sync, build a PSBT, sign it, broadcast it |
| `vault-create --unlock-height <h>` | Stretch: derive a CLTV-timelocked P2WSH address with raw `rust-bitcoin` |
| `vault-spend --outpoint <txid:vout> --amount <sats> --wif <wif> --redeem-script <hex> --unlock-height <h> --to <addr>` | Stretch: manually sign and broadcast a spend from that vault |

All node/wallet settings (`--network`, `--rpc-url`, `--rpc-cookie`, `--rpc-user`,
`--rpc-pass`, `--db-path`, `--mnemonic`, `--account`, `--script-type`) are also
readable from `.env` (see `.env.example`) or environment variables, so you don't have
to repeat flags on every invocation.

## Project structure

- `src/config.rs` -- the `clap` CLI: global node/wallet options plus subcommands.
- `src/keys.rs` -- BIP39 mnemonic generation/loading and BIP84 (`wpkh`) / BIP86 (`tr`)
  descriptor derivation, for external (`m/.../0/*`) and internal (`m/.../1/*`)
  keychains.
- `src/walletdb.rs` -- opens/creates the `bdk_wallet` `Wallet` against its SQLite
  store, and syncs it against bitcoind via `bdk_bitcoind_rpc::Emitter` (blocks, then
  mempool).
- `src/node.rs` -- builds the `bitcoincore-rpc` client (cookie, user/pass, or none).
- `src/manual.rs` -- the raw `rust-bitcoin` CLTV vault (stretch goal).
- `src/main.rs` -- command dispatch: `send` builds/signs/broadcasts a PSBT inline.

### Why this descriptor structure

The wallet derives two BIP84 (native SegWit, `wpkh`) descriptors from one BIP39
mnemonic: `m/84'/1'/0'/0/*` for the external (receive) keychain and
`m/84'/1'/0'/1/*` for the internal (change) keychain (coin type `1'` since this is a
test network; account `0'`, configurable via `--account`). Keeping receive and change
on separate branches is the standard reason to use two descriptors instead of one --
it lets a wallet (or a watch-only observer holding only the public descriptors) tell
"funds I received" apart from "change I sent to myself" without leaking that
distinction into a single address space. `--script-type tr` switches both to BIP86
Taproot (`m/86'/...`) as a stretch option; the account/coin-type logic is shared.

**A deliberate security choice:** `Wallet::create`/`Wallet::load` are given the
*private* descriptors (with the `tprv`/`vprv` embedded) on every run, re-derived from
the mnemonic in memory each time -- but only the corresponding *public* descriptors
are ever persisted into `wallet.sqlite` (see `derive_descriptors` in `src/keys.rs`,
which returns both forms from a single `into_wallet_descriptor` call). The SQLite
store ends up holding chain state (UTXOs, tx graph, keychain indices) and nothing
that lets you spend; the mnemonic in `.env` is the only thing that does.

## Libraries used, and why

- **`bdk_wallet`** -- descriptor parsing/derivation, keychain-aware address
  generation, UTXO/balance tracking, coin selection, PSBT construction (`build_tx`),
  and signing (`wallet.sign`). This is the part of the assignment ("track UTXOs",
  "generate addresses from a descriptor", "persist wallet state") that's exactly what
  BDK is for, so there was no reason to reimplement it by hand.
- **`bdk_bitcoind_rpc`** -- bridges bitcoind's block/mempool data into BDK's chain
  graph (`Emitter`), so `sync` can replay confirmed blocks and pick up unconfirmed
  mempool transactions without an external indexer (Electrum/Esplora).
- **`bitcoincore-rpc`** (used here via `bdk_bitcoind_rpc`'s re-export, which is the
  same crate/version, not a fork) -- the actual RPC client: `getblockchaininfo` on
  startup, block-by-block sync inside `Emitter`, and `send_raw_transaction` to
  broadcast.
- **`rust-bitcoin`** directly (`src/manual.rs`) -- for the vault stretch goal, see
  below.
- **`clap`** (derive + env) -- CLI parsing that also reads from `.env`/environment,
  so RPC and wallet settings don't need to be retyped on every command.
- **`dotenvy`** -- loads `.env` at startup.
- **`anyhow`** -- error handling glue across the above.

## Stretch goals attempted

- **Multiple descriptor types**: `--script-type wpkh` (default, BIP84) vs `--script-type
  tr` (BIP86 Taproot) -- both were exercised end-to-end against a live regtest node
  (address generation, funding, syncing, and a signed/broadcast send).
- **Raw `rust-bitcoin` instead of BDK`** (`vault-create` / `vault-spend`): `bdk_wallet`'s
  `TxBuilder`/`wallet.sign` can only satisfy scripts that come from one of the
  wallet's own descriptors -- there's no signer for an arbitrary custom witness
  script. A CLTV-timelocked single-key vault (`<height> OP_CLTV OP_DROP <pubkey>
  OP_CHECKSIG`, as P2WSH) is exactly that: `vault-create` builds the redeem script and
  its address by hand with `bitcoin::script::Builder`; `vault-spend` computes the
  BIP143 sighash with `SighashCache::p2wsh_signature_hash`, signs it directly with
  `secp256k1`, and assembles the `[signature, redeem_script]` witness manually, since
  there's no descriptor for BDK to derive a signer from. Verified on regtest: a spend
  attempted before the unlock height is rejected by the node (`"non-final"`), and
  succeeds once the chain reaches it.

## Known limitations / what I'd improve with more time

- `send` uses BDK's default coin selection and always syncs before building a
  transaction; there's no explicit UTXO selection flag.
- No fee bumping (RBF) or replace-by-fee command.
- The vault demo's spend fee is a flat 500 sats rather than a vsize-based estimate.
- `sync` always rescans from height 0 through `Emitter`'s block-by-block walk (fine on
  regtest; on a longer-lived chain you'd want the wallet's persisted checkpoint to
  short-circuit that, which `Emitter::new` already takes a `wallet_tip` for -- it's
  wired up, just not benchmarked against a large chain here).
- No automated test suite; this was validated by hand against a live regtest node
  (see the command sequence under Setup) rather than with `cargo test`.
