# Assignment: Building a Bitcoin Wallet in Rust

## Goal

Build a functioning Bitcoin wallet in Rust (regtest) that demonstrates you can use the libraries covered in class effectively.

## Project Status

The project currently provides a compiling Cargo workspace with wallet
initialization, address derivation, and Bitcoin Core synchronization
implemented. It can generate or import a BIP39 mnemonic, derive BIP84
receiving/change addresses, persist/reopen the wallet through SQLite, verify
the configured Bitcoin Core RPC connection and network, persist confirmed
chain and mempool updates, calculate balances, inspect wallet UTXOs, and build,
sign, broadcast, and persist outgoing transactions.

## Architecture

```text
bin/
└── src/main.rs                    # Minimal executable entry point
crates/
└── wallet/                        # Cohesive application library
    └── src/
        ├── lib.rs                 # Startup and command orchestration
        ├── cli.rs                 # Commands and argument validation
        ├── config.rs              # Network, storage, and RPC configuration
        ├── error.rs               # Shared typed errors
        ├── logging.rs             # Structured logging configuration
        ├── types.rs               # Wallet domain/output types
        ├── core/
        │   ├── keys.rs            # Key creation and address derivation
        │   ├── sync.rs            # Chain sync, balances, and UTXOs
        │   └── transactions.rs    # Transaction building and signing
        ├── node/
        │   └── bitcoin_core.rs    # Bitcoin Core RPC adapter and trait
        └── persistence/
            └── sqlite.rs          # BDK SQLite persistence boundary
```

The dependency flow is simply `muf_wallet` → `wallet`. The executable contains
only `main`, while responsibilities remain separated by focused internal modules.
This keeps navigation and refactoring simple without premature crate boundaries.

Potential BDK contribution opportunities discovered during implementation are
tracked in [`docs/bdk-friction-log.md`](docs/bdk-friction-log.md).

## Development Quick Start

```bash
cp .env.example .env
cargo run -p muf_wallet -- --help
cargo test
```

The default network is `regtest`. Real RPC credentials and seed material must
remain outside version control. The application loads `.env` automatically when
present; command-line options still take precedence over environment values.

### Initialize the wallet

Generate a new 12-word recovery phrase and initialize the SQLite wallet:

```bash
cargo run -p muf_wallet -- init
```

The recovery phrase is printed only when a new phrase is generated. Back it up
before continuing. To import an existing phrase, place it in the untracked
`.env` file instead of putting it in shell history:

```env
MUF_MNEMONIC="your twelve recovery words go here"
```

The SQLite database contains public descriptors and wallet state, not the
mnemonic. This educational regtest wallet therefore needs the backed-up phrase
again when restoring signing access.

### Test initialization manually

Use a disposable directory so the test does not touch your main wallet:

```bash
MUFASA_TEST_DATA_DIR="$(mktemp -d)"
cargo run -p muf_wallet -- --network regtest --data-dir "$MUFASA_TEST_DATA_DIR" init
```

Verify that the database was created and contains BDK wallet state:

```bash
test -f "$MUFASA_TEST_DATA_DIR/wallet.sqlite3"
sqlite3 "$MUFASA_TEST_DATA_DIR/wallet.sqlite3" ".tables"
sqlite3 "$MUFASA_TEST_DATA_DIR/wallet.sqlite3" \
  "SELECT network, descriptor, change_descriptor FROM bdk_wallet;"
```

Run the same initialization command again. It should fail safely with `a wallet
is already initialized`, demonstrating that persisted state is detected instead
of overwritten.

### Derive addresses

Derive and persist the next receiving or change address:

```bash
cargo run -p muf_wallet -- address
cargo run -p muf_wallet -- address --change
```

Each command reopens the existing SQLite wallet and persists the revealed
derivation index before displaying the address.

### Check Bitcoin Core

Verify that the configured RPC endpoint is reachable and serves the expected
network:

```bash
cargo run -p muf_wallet -- node-health
```

For Polar, copy the Bitcoin Core node's RPC host, port, username, and password
into the corresponding `MUF_RPC_*` values in the private `.env` file.

### Synchronize the wallet

After deriving at least one receiving address, synchronize the wallet with the
configured Bitcoin Core node:

```bash
cargo run -p muf_wallet -- sync
```

The command reopens the wallet, verifies the node network, streams blocks from
the wallet's latest persisted checkpoint, applies current mempool changes, and
persists the resulting BDK changeset to SQLite. A later invocation resumes from
that checkpoint instead of processing the same blocks again.

### Inspect balances and UTXOs

Synchronize and then inspect the wallet:

```bash
cargo run -p muf_wallet -- balance
cargo run -p muf_wallet -- utxos
```

`balance` separates confirmed, trusted pending, untrusted pending, immature,
spendable, and total funds. `utxos` displays each output's outpoint, value,
receiving/change derivation path, confirmation status, coinbase maturity,
locked state, and spendability. Both commands synchronize with Bitcoin Core by
default. Use `--offline` to inspect the latest state already saved in SQLite:

```bash
cargo run -p muf_wallet -- balance --offline
cargo run -p muf_wallet -- utxos --offline
```

### Send bitcoin

Keep the wallet's mnemonic in the private `.env` file so BDK can load its
signing keys:

```env
MUF_MNEMONIC="your twelve recovery words go here"
```

Send an exact satoshi amount to an address on the configured network:

```bash
cargo run -p muf_wallet -- send \
  --to bcrt1q... \
  --amount 100000 \
  --fee-rate 2
```

Before coin selection, `send` synchronizes the wallet. BDK then selects usable
UTXOs, calculates a change output and fee, builds and signs a PSBT, and
finalizes the transaction. The application persists any new change derivation
index, broadcasts through Bitcoin Core, records the outgoing transaction as
unconfirmed, persists it, and prints its transaction ID and fee.

### CLI

```text
muf_wallet init
muf_wallet address [--change]
muf_wallet node-health
muf_wallet sync
muf_wallet balance [--offline]
muf_wallet utxos [--offline]
muf_wallet send --to <ADDRESS> --amount <SATS> [--fee-rate <SAT/VB>]
```

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
