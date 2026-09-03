# Work Done — Week 6

## Project Summary

Week 6 is a different shape from Weeks 1–5: instead of ten `todo!()` labs graded against a fixed
test suite, the assignment is open-ended — "build a functioning Bitcoin wallet in Rust (regtest)".
This document tracks what was built and, more importantly, how it was verified.

## Repository

- **GitHub URL**: https://github.com/nzubepolycap-hub/rust-for-bitcoin-2.0
- **Branch**: `rust-for-bitcoin-6.0`
- **Local path**: `/home/blackghost/Documents/rust-for-bitcoin-2.0/rust-for-bitcoin-2.0/rfb_labs_week_6`
- **Upstream assignment**: https://github.com/thebuidl-grid/rust-for-bitcoin-2.0/tree/main/rfb_labs_week_6

## What was accomplished

### 1. Fetched the assignment scaffold

`rfb_labs_week_6/` didn't exist in this fork yet. Pulled the unmodified starter (`Cargo.toml`,
`src/main.rs`, `Readme.md`) from upstream `thebuidl-grid/rust-for-bitcoin-2.0`. The upstream brief
was kept, renamed to `ASSIGNMENT.md` (to avoid a case-only filename collision with the submission
`README.md` this session wrote, on filesystems where that matters).

### 2. Verified the exact crate APIs before writing any code

This assignment isn't graded against a fixed test suite, so there was no compiler-enforced
contract to write against. Rather than write against remembered/assumed BDK APIs, every API used
below was checked against the real published source at the exact locked version (`bdk_wallet
3.1.0`, `bdk_bitcoind_rpc 0.22.0`, `bitcoincore-rpc 0.19.0`, `bitcoin 0.32.102`, `bip39 2.2.2`) —
mostly BDK's own `examples/bitcoind_rpc.rs` and doc-tests in `src/wallet/mod.rs`,
`src/descriptor/template.rs` and `src/wallet/params.rs` — before it went into `src/`.

### 3. Built the wallet (`src/`)

| File | Responsibility |
|---|---|
| `config.rs` | `clap` CLI, every flag env-var-backed (`DESCRIPTOR`, `RPC_URL`, ...), mirroring BDK's own example |
| `keys.rs` | Fresh BIP39 mnemonic → BIP32 master `Xpriv` → `wpkh()`/`tr()` descriptor strings (BIP84/BIP86) |
| `wallet_store.rs` | `init` (generate + create + persist) and `open` (load) the BDK wallet against its SQLite persister |
| `chain.rs` | `bitcoincore-rpc` client construction; block/mempool sync via `bdk_bitcoind_rpc::Emitter` |
| `main.rs` | CLI dispatch: `init`, `address`, `change-address`, `sync`, `balance`, `utxos`, `send`, `mine` |

Covers every minimum requirement: key generation from a descriptor, separate external/internal
keychains, UTXO/balance tracking, SQLite persistence across restarts, and a real
build-sign-broadcast `send` connected to Bitcoin Core over `bitcoincore-rpc` (via
`bdk_bitcoind_rpc`'s re-export of it).

### 4. Two bugs caught by actually running it, not just compiling it

Both would have shipped silently if this had stopped at "it builds":

- **`SignOptions::default()` refused to sign at all** (`partial_sigs: 0`) because `TxBuilder`
  only attaches `witness_utxo` per input, and `trust_witness_utxo` defaults to `false` as a
  SegWit-bug mitigation for *foreign* PSBTs. Since the PSBT here is built by this same wallet from
  its own synced chain data, set `trust_witness_utxo: true` explicitly (documented inline in
  `main.rs`).
- **Loaded wallets had zero registered signers** (`get_signers(..).signers().len() == 0`) even
  though `.env`'s `DESCRIPTOR` embeds the `tprv`. `LoadParams::descriptor()` only *checks* a
  provided descriptor against the persisted one by default — `bdk_wallet::Wallet::load()` needs an
  explicit `.extract_keys()` call to actually pull private keys out of it into signers. Found by
  reading `bdk_wallet-3.1.0`'s `src/wallet/params.rs` after the first `send` failed.

### 5. End-to-end verification against a real regtest node

Bitcoin Core v27.1.0 was already present on this machine at `~/bitcoin-node/` (regtest-configured
`~/.bitcoin/bitcoin.conf`), so nothing was mocked or simulated:

```
cargo build          # clean
cargo fmt --check     # clean
cargo clippy --all-targets -- -D warnings   # clean, zero warnings
```

Then, against a live `bitcoind -regtest`:

1. `init` → fresh mnemonic, `wpkh` descriptors, `wallet.sqlite` created, `.env` written.
2. `mine --blocks 101` → wallet's own address funded via `generatetoaddress`, balance synced.
3. `balance` / `address` / `change-address` / `utxos` — all read back correctly from a **fresh
   process** each time (persistence proof, since nothing is kept in memory between commands).
4. `send --to <own new address> --amount 1234567 --fee-rate 2` → broadcast a real transaction;
   independently confirmed via `bitcoin-cli getrawtransaction` that the txid, output amount, and
   destination address match exactly.
5. `mine --blocks 1` → the send confirmed; `utxos` afterward shows the payment output on the
   External keychain and the change output on the Internal keychain, at the correct derivation
   indices.
6. Repeated steps 1–4 with `init --kind tr` in an isolated directory to confirm the Taproot
   (BIP86) descriptor path also signs and broadcasts correctly (Schnorr key-path spend), proving
   the stretch goal actually works rather than just compiles.

Full transcripts (with real txids/addresses, no key material) are in `README.md` under
"Evidence: a real regtest transaction".

### 6. Git workflow

- Reused this fork's per-week branch convention: continued on `rust-for-bitcoin-6.0`. That branch
  name already existed locally/remotely but pointed at a stale pre-Week-3 commit that was fully
  contained in the current history (`git merge-base --is-ancestor` confirmed no unique commits
  would be lost), so it was fast-forwarded to the tip of `rust-for-bitcoin-5.0` before starting
  Week 6's work — no history was discarded.
- `.gitignore` scoped to `rfb_labs_week_6/` excludes `.env` and `*.sqlite*`; `.env.example`
  documents every variable with no real values.
