# Assignment: Building a Bitcoin Wallet in Rust

## Goal

Build a functioning Bitcoin wallet in Rust (regtest) that demonstrates you can use the libraries covered in class effectively.

## Implementation

This repository contains a small command-line wallet in `src/main.rs`. It generates a fresh BIP32 master key on `init`, stores it in a local SQLite database, derives separate external and internal native SegWit keychains, prints descriptors, and scans Bitcoin Core's UTXO set with `scantxoutset`.

The default network is regtest. The database stores the encrypted-by-filesystem-equivalent local wallet state for this classroom exercise, address metadata, and discovered UTXOs. Do not use this storage design for production funds.

Libraries used:

- `bitcoin` derives keys, paths, compressed public keys, addresses, amounts, and network-specific encodings.
- `bitcoincore-rpc` connects to Bitcoin Core and calls blockchain information and descriptor UTXO scanning RPCs.
- `rusqlite` persists the wallet key, keychain counters, addresses, and UTXO set locally.
- `clap` provides the CLI and environment-variable configuration.
- `rand` generates fresh seed material at initialization.

## Running

From this directory:

```bash
cargo run -- --database wallet.db init
cargo run -- --database wallet.db address external
cargo run -- --database wallet.db address internal
cargo run -- --database wallet.db show-descriptor
cargo run -- --database wallet.db balance
```

Start a regtest node with RPC enabled, then configure credentials when needed:

```bash
export BITCOIN_RPC_URL=http://127.0.0.1:18443
export BITCOIN_RPC_USER=rpcuser
export BITCOIN_RPC_PASSWORD=rpcpassword
cargo run -- --database wallet.db sync
```

`sync` scans descriptor ranges 0 through 99 and persists discovered UTXOs. The `send` command validates the destination and amount and requires the descriptors to be imported into a Bitcoin Core wallet before wallet-backed PSBT creation and signing. This keeps private-key handling inside Core rather than pretending an offline placeholder is a broadcast transaction.

## Design and limitations

The wallet uses `wpkh` descriptors with BIP84-style external branch 0 and internal branch 1. SQLite makes counters and discovered UTXOs survive process restarts. Descriptor checksums, encrypted-at-rest secrets, gap-limit expansion, confirmations, fee selection, PSBT signing, and automatic Core wallet creation are still improvements for a production implementation. The assignment is intentionally limited to regtest/testnet and does not handle mainnet funds.

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