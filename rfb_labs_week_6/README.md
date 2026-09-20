
# RFB Week 6 — A Minimal Regtest Bitcoin Wallet

A command-line Bitcoin wallet built in Rust, using BDK for wallet logic and
Bitcoin Core (via Polar) as the connected node. Regtest.

## Setup

### 1. A running regtest node

This project was built and tested against [Polar](https://lightningpolar.com/),
which runs Bitcoin Core (and optionally Lightning nodes) in Docker containers.

1. Open Polar, create a network with at least one Bitcoin Core node, and start it.
2. In Polar, click the Bitcoin Core node → "Connect" tab to find its RPC port
   and credentials.

Any regtest-capable Bitcoin Core node works — Polar is just the easiest way
to get one running locally.

### 2. Configure `.env`

Copy the example file and fill in your node's actual RPC details:

```bash
cp .env.example .env
```

```env
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=polaruser
BITCOIN_RPC_PASSWORD=polarpass
BITCOIN_NETWORK=regtest

# Optional — pin a specific wallet mnemonic. If unset, the wallet generates
# one automatically on first run and reuses it from wallet_seed.txt.
# WALLET_MNEMONIC=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

`.env` is gitignored and must never be committed — it's local configuration only.

### 3. Build

```bash
cargo build --release
```

## Usage

```bash
cargo run -- <COMMAND>
```

Note the `--` before the subcommand — without it, Cargo tries to interpret
flags like `send` as arguments to `cargo run` itself.

| Command | Description |
|---|---|
| `address` | Reveal the next unused external (receive) address |
| `address --internal` | Reveal the next unused internal (change) address |
| `balance` | Sync with the node, then print a full balance breakdown |
| `sync` | Sync wallet state against the node and report the new checkpoint |
| `send <ADDRESS> <AMOUNT_BTC>` | Sync, then build, sign, and broadcast a transaction |
S
### Examples

Get a receive address:
```bash
cargo run -- address
# bcrt1q959gmflkzeaply5at94epu6u6rf0gcxx5mwzx6
```

Fund the wallet (regtest only — mine directly to your address):
```bash
docker exec <bitcoind-container-name> bitcoin-cli -regtest \
  -rpcuser=polaruser -rpcpassword=polarpass \
  generatetoaddress 101 <your-address>
```
101 blocks are needed because Bitcoin requires 100 confirmations before a
coinbase reward becomes spendable — the extra block is the one carrying the
reward itself.

Check balance:
```bash
cargo run -- balance
# Balance breakdown:
#   confirmed:          98.99999859 BTC
#   trusted pending:    0 BTC
#   untrusted pending:  0 BTC
#   immature:           3525.00000141 BTC
#   total:              3624.00000000 BTC
```

Send funds:
```bash
cargo run -- send bcrt1q5t2ee4j3vlap5u9p77f3wr544j0y0340yskkmj 0.1
# Sync complete: applied 0 block(s).
# Broadcast! txid: 196da3f3e34b15df028f0c0d5cd22548171a9be27ae812adcc5bc4885d869768
```

Verify a broadcast transaction directly against the node:
```bash
docker exec <bitcoind-container-name> bitcoin-cli -regtest \
  -rpcuser=polaruser -rpcpassword=polarpass \
  getrawtransaction 196da3f3e34b15df028f0c0d5cd22548171a9be27ae812adcc5bc4885d869768 true
```

## Architecture

### Project structure

```
src/
├── main.rs     — CLI entry point, dispatches subcommands
├── cli.rs      — clap argument/subcommand definitions
├── config.rs   — loads RPC and network config from .env
├── seed.rs     — resolves a stable BIP39 mnemonic across restarts
├── node.rs     — bitcoincore-rpc client + connectivity check
├── wallet.rs   — BDK wallet creation/loading, descriptor building, sync
└── send.rs     — transaction building, signing, and broadcasting
```

### Wallet identity and persistence

The wallet's key material comes from a BIP39 mnemonic, resolved in this
priority order (`seed.rs`):

1. `WALLET_MNEMONIC` in `.env`, if set.
2. A previously generated mnemonic saved locally at `wallet_seed.txt`.
3. Otherwise, a fresh mnemonic is generated and saved to `wallet_seed.txt`
   for reuse on every future run.

This is what makes the wallet's identity — and therefore its addresses and
UTXOs — stable across restarts, without requiring any manual setup.

### Descriptor structure

The wallet uses a single-sig native SegWit (`wpkh`) descriptor, following
BIP84 convention, with separate external (receive) and internal (change)
keychains derived from one master key:

```
wpkh(<master xpriv>/84h/1h/0h/0/*)   — external, receive addresses
wpkh(<master xpriv>/84h/1h/0h/1/*)   — internal, change addresses
```

`wpkh` was chosen as the baseline because it's the simplest widely-used
modern script type, and matches the primary address format covered in
Week 5. Coin type `1h` is used for testnet/regtest per convention (`0h` is
reserved for mainnet).

Wallet state — keychain metadata, known UTXOs, transaction history — is
persisted to a local SQLite database (`wallet.sqlite`, via BDK's
`rusqlite` backend) after every mutating operation (revealing an address,
syncing, sending), so closing and reopening the wallet never requires
re-deriving or re-syncing from scratch.

## Library usage and rationale

| Library | Used for | Why |
|---|---|---|
| `bdk_wallet` | Descriptor parsing, keychain management, UTXO tracking, balance calculation, transaction building and signing, SQLite persistence | Purpose-built for exactly this — reimplementing coin selection, PSBT construction, and signing correctly by hand in raw `rust-bitcoin` would duplicate a large amount of well-tested logic |
| `bdk_bitcoind_rpc` | Bridges BDK's chain-update model to a Bitcoin Core RPC connection, emitting blocks and mempool events the wallet can apply | This is the missing link between "a BDK wallet" and "a wallet that actually knows what happened on-chain" |
| `bitcoincore-rpc` | Node connectivity check, transaction broadcast | Direct, simple RPC calls that don't need to go through BDK's abstractions |
| `bitcoin` | Shared types (`Network`, `Amount`, `Address`) used across the app | The foundational crate everything else in the Bitcoin Rust ecosystem builds on |
| `bip39` | Mnemonic generation, parsing, and seed derivation | Same crate used in Week 5's HD wallet labs |
| `clap` | CLI argument/subcommand parsing | Standard, well-documented, and avoids hand-rolling argument parsing |
| `dotenvy` | Loading `.env` for local configuration | Keeps RPC credentials and wallet config out of source code, per the assignment's constraint |
| `anyhow` | Application-level error handling with context | Appropriate for a binary/application (as opposed to a library with its own error enum) — gives clear, contextual error messages via `?` without a lot of boilerplate |

No raw `rust-bitcoin`-only code path (bypassing BDK) was needed for the
minimum requirements — BDK's `Wallet` and `bdk_bitcoind_rpc`'s `Emitter`
covered everything from address derivation through signing.

## Known limitations

- **Address reuse in a specific edge case:** when a transaction is sent to
  a freshly revealed internal (change) address in the same run where that
  address was just revealed but not yet synced, BDK's automatic change
  output can land on that same address — because "revealed" and "observed
  as used" are tracked separately, and the wallet hasn't yet seen the
  transaction that would mark the address used. This was observed directly
  while testing: a deliberate 1 BTC send to a freshly revealed internal
  address landed alongside the automatic change output on that identical
  address. A more careful implementation would check for this collision
  before finalizing a transaction, or use `TxBuilder::drain_to` to control
  the change destination explicitly rather than relying on the default.
- **Single-signature only.** No multisig or Taproot support — `wpkh` was
  chosen as the baseline for simplicity; comparing descriptor types is
  listed as a stretch goal not attempted here.
- **No coin selection strategy beyond BDK's default.** For a wallet with
  many small UTXOs (as this one accumulates from repeated `generatetoaddress`
  calls during testing), a more deliberate selection strategy could reduce
  fees or UTXO fragmentation.
- **No automated tests.** Given the scope and time available, correctness
  was verified manually against a live regtest node rather than with a
  test suite exercising each function in isolation.
- **Fixed fee estimation** relies on BDK's defaults rather than querying
  the node for a current feerate — fine on regtest, would need attention
  for testnet/mainnet use.

## Verified working transaction

Txid: `196da3f3e34b15df028f0c0d5cd22548171a9be27ae812adcc5bc4885d869768`

Confirmed present in Bitcoin Core's mempool via `getrawtransaction`, sending
0.1 BTC from this wallet to a separate regtest address, with the correct
SegWit witness structure and a change output returned to the wallet.