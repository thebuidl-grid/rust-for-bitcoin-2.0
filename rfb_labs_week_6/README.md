# Week 6 Assignment: Bitcoin Rust Crates Wallet

A fully functional, modular Bitcoin CLI wallet built in Rust for `regtest` and `testnet`. This wallet demonstrates the combined use of `rust-bitcoin`, `bdk_wallet` (with SQLite persistence), and `bitcoincore-rpc` / `bdk_bitcoind_rpc`.

---

## Architecture & Project Structure

```
rfb_labs_week_6/
├── Cargo.toml          # Dependency manifest
├── README.md           # Documentation & design rationale
├── .env.example        # Environment variable configuration template
└── src/
    ├── main.rs         # Application entrypoint & CLI subcommand router
    ├── config.rs       # CLI definitions (clap) & env config (dotenvy)
    ├── keys.rs         # BIP39 seed generation & HD descriptor derivation (wpkh & tr)
    ├── wallet.rs       # BDK Wallet manager with SQLite persistence (rusqlite)
    ├── node.rs         # Bitcoin Core RPC client, block/mempool sync engine & broadcast
    └── raw_bitcoin.rs  # Demonstration of low-level rust-bitcoin script/PSBT manipulation
```

---

## Key Features

1. **Key Generation & Descriptor Derivation (`src/keys.rs`)**
   - Generates 12-word BIP39 mnemonic phrases using cryptographically secure entropy.
   - Derives HD master keys and outputs external (`/0/*`) and internal change (`/1/*`) descriptors.
   - Supports both **Native SegWit** (`wpkh`, BIP84) and **Taproot** (`tr`, BIP86) descriptors.

2. **Wallet State & SQLite Persistence (`src/wallet.rs`)**
   - Wraps `bdk_wallet::PersistedWallet` with a `rusqlite` database backend (`wallet.sqlite`).
   - Tracks received addresses, change addresses, UTXOs, and balance breakdowns (confirmed, pending, immature).
   - Survives process restarts without losing state or needing full rescans.

3. **Node Integration & Sync Engine (`src/node.rs`)**
   - Connects to Bitcoin Core nodes via JSON-RPC.
   - Uses `bdk_bitcoind_rpc::Emitter` for block-by-block and mempool syncing.
   - Broadcasts signed transactions directly to the network.

4. **Raw `rust-bitcoin` vs BDK Demonstration (`src/raw_bitcoin.rs`)**
   - Demonstrates building custom `OP_RETURN` metadata payloads using `rust-bitcoin` script builders.
   - Shows raw PSBT inspection and low-level data manipulation.

---

## Library Usage Justification

| Library | Role in Project | Why Used Here |
|---|---|---|
| `bdk_wallet` | High-level Wallet Logic & Persistence | Handles descriptor-based wallet tracking, UTXO management, coin selection, PSBT construction/signing, and SQLite persistence. |
| `bdk_bitcoind_rpc` / `bitcoincore-rpc` | Node Synchronization & RPC | Connects to Bitcoin Core, emits block/mempool updates, and broadcasts transactions. |
| `rust-bitcoin` | Low-Level Bitcoin Primitives | Used for master key derivation, BIP32 derivation paths, raw script building (`OP_RETURN`), and PSBT serialization. |
| `bip39` | Seed Phrase Generation | Generates 12-word mnemonics and converts phrases to 512-bit seed bytes. |
| `clap` / `dotenvy` | Configuration & CLI Interface | Command-line parsing with environment variable overrides for clean configuration. |

---

## Scenario: Why & When to Use Raw `rust-bitcoin` Over BDK

### Scenario
BDK is designed for standard HD wallet payments (Native SegWit, Taproot, Multisig). However, when building **custom protocol layers** (such as proof-of-existence anchors, timestamping services, Ordinals/Runes, or custom smart contract scripts), high-level BDK abstractions do not provide direct methods to construct arbitrary non-standard outputs like `OP_RETURN` payloads.

In these cases, we reach for raw `rust-bitcoin` directly to build `ScriptBuf` outputs using `Builder::new().push_opcode(OP_RETURN).push_slice(...)`.

### Code Example (from `src/raw_bitcoin.rs`)
```rust
use bdk_wallet::bitcoin::blockdata::opcodes;
use bdk_wallet::bitcoin::blockdata::script::Builder;
use bdk_wallet::bitcoin::script::PushBytesBuf;
use bdk_wallet::bitcoin::{Amount, TxOut};

pub fn build_op_return_output(message: &[u8]) -> anyhow::Result<TxOut> {
    let mut push_bytes = PushBytesBuf::new();
    push_bytes.extend_from_slice(message)?;

    let script = Builder::new()
        .push_opcode(opcodes::all::OP_RETURN)
        .push_slice(push_bytes)
        .into_script();

    Ok(TxOut {
        value: Amount::ZERO,
        script_pubkey: script,
    })
}
```

---

## Setup & Running Instructions

### 1. Requirements
- Rust toolchain (`cargo`, edition 2021/2024).
- Optional: A running `bitcoind` node in regtest mode.

### 2. Quickstart Commands

#### Generate New Mnemonic & Descriptors (Native SegWit)
```bash
cargo run -- generate-mnemonic
```

#### Generate Taproot Mnemonic & Descriptors
```bash
cargo run -- --descriptor-type taproot generate-mnemonic
```

#### Get a New Receiving Address
```bash
cargo run -- --mnemonic "your 12 word seed phrase here..." address
```

#### Get a Change Address
```bash
cargo run -- --mnemonic "your 12 word seed phrase here..." address --change
```

#### Check Wallet Balance
```bash
cargo run -- --mnemonic "your 12 word seed phrase here..." balance
```

#### List UTXOs
```bash
cargo run -- --mnemonic "your 12 word seed phrase here..." utxos
```

#### Sync Wallet with Bitcoin Core RPC Node
```bash
cargo run -- --rpc-url "http://127.0.0.1:18443" --rpc-user admin --rpc-pass password --mnemonic "your seed phrase" sync
```

#### Construct, Sign, and Broadcast a Transaction
```bash
cargo run -- --mnemonic "your seed phrase" send --recipient "bcrt1q..." --amount-sats 50000 --broadcast
```

#### Run Raw `rust-bitcoin` Script Demonstration
```bash
cargo run -- raw-demo --message "Hello Bitcoin"
```

---

## Proof of Working Functionality & Test Verification

### Unit Tests
Run automated unit tests covering key generation, Taproot descriptors, and raw OP_RETURN script construction:
```bash
cargo test
```

Expected Output:
```text
running 3 tests
test keys::tests::test_generate_and_derive_taproot ... ok
test keys::tests::test_generate_and_derive_wpkh ... ok
test raw_bitcoin::tests::test_op_return_script ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## Known Limitations & Future Improvements
1. **Coin Selection Control**: Currently uses BDK's default coin selection algorithm. Future versions could expose manual UTXO locking/selection flags.
2. **Async RPC/Esplora Backend**: The node client currently uses synchronous HTTP RPC. Adding async Esplora or Electrum client support would enable wallet sync over public networks without requiring a full local `bitcoind` node.
