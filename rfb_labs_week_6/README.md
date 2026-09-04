# Week 6: `rfbwallet` - a regtest Bitcoin wallet in Rust

A single-signature HD wallet that runs against `bitcoind` on regtest (or testnet /
signet). It derives from a descriptor, tracks its own UTXOs, keeps state in
SQLite across restarts, and builds, signs and broadcasts real transactions
through a Bitcoin Core node.

- Wallet logic (descriptors, keychains, coin selection, PSBT, persistence): **BDK**
- Node I/O (connect, sync, broadcast, mine): **`bitcoincore-rpc`** (directly, and
  via `bdk_bitcoind_rpc`)
- Raw transaction primitives (decode / hand-build): **`rust-bitcoin`**

## What it does

| Requirement | Where |
|---|---|
| Generate or import keys, derive from a descriptor | `src/keys.rs`, `rfbwallet init` |
| External (receive) and internal (change) keychains | descriptor pair, `src/wallet.rs` |
| Track UTXOs and calculate balance | `rfbwallet utxos`, `rfbwallet balance` |
| Persist wallet state locally (SQLite) | `bdk_wallet` `rusqlite` persister, `src/wallet.rs` |
| Construct, sign, broadcast a transaction | `src/tx.rs`, `rfbwallet send` |
| Connect to a Bitcoin node | `src/node.rs`, `src/sync.rs` |

Stretch goals attempted and working: a full CLI, `wpkh` vs `tr` (Taproot)
descriptors, explicit manual coin selection (`--utxo`), `thiserror` error
handling plus `tracing` logging, and a raw `rust-bitcoin` section with a runnable
example (`rfbwallet raw-demo`).

## Setup

### 1. Prerequisites

- Rust (stable, edition 2024 - tested on 1.96)
- Bitcoin Core (tested on v31). The Core snap exposes the binaries as
  `bitcoin-core.daemon` and `bitcoin-core.cli`.

### 2. Start a regtest node

Either use your own `bitcoind`, or the helper script:

```bash
cd rfb_labs_week_6

# with the Bitcoin Core snap:
BITCOIND=bitcoin-core.daemon BITCOIN_CLI=bitcoin-core.cli ./scripts/regtest.sh start
# prints RPC_URL and RPC_COOKIE for the next step
```

### 3. Configure the wallet

```bash
cp .env.example .env
```

Edit `.env` and set `RPC_COOKIE` to the path the script printed (or set
`RPC_USER` / `RPC_PASS` instead). Leave `MNEMONIC` blank.

### 4. Create the wallet

```bash
cargo run -- init          # or: cargo run -- init --taproot
```

`init` generates a BIP39 mnemonic, writes it back into `.env` (which is
gitignored), derives the descriptors, and creates `wallet.sqlite`. To **import**
an existing wallet instead, paste its mnemonic into `MNEMONIC=` before running
`init`.

### 5. Use it

```bash
cargo run -- address                       # new receiving address
cargo run -- mine 101                       # regtest only: fund the wallet
cargo run -- sync                           # pull blocks + mempool from the node
cargo run -- balance
cargo run -- utxos
cargo run -- send --to <address> --amount-sat 100000000
cargo run -- send --to <address> --amount-sat 50000000 --fee-rate 5 --utxo <txid:vout>
cargo run -- send --to <address> --drain    # sweep everything
cargo run -- node-info                      # proves the RPC connection
cargo run -- descriptors                    # public descriptors
cargo run -- raw-demo --decode <raw_tx_hex> # stretch: raw rust-bitcoin
```

Set `RUST_LOG=debug` for verbose logging.

## Proof of a working transaction (regtest)

```
$ rfbwallet send --to bcrt1q8un3mg4240nxcawdlkm2mr9v0zeg06tvukwdfl --amount-sat 100000000
broadcast ok
txid: 9852fd9eff258fc6decda9cf1c21d390e5ee02b290056ea340a6d148ae04d635
sent: 1 BTC
fee:  0.00000141 BTC
raw:  020000000001019a431e46eb333a5fa26fad813035c3a46cdc8d2ee3ab80bb5011cb4ac8b36333...

$ bitcoin-core.cli -regtest getrawtransaction 9852fd9e...d635 true
  "txid": "9852fd9eff258fc6decda9cf1c21d390e5ee02b290056ea340a6d148ae04d635",
  "vsize": 141,
  "blockhash": "1dfe9b0786ba82be58f5d201e5d4fb653ec79c5d338f24c2ffa17668cd946d72",
  "confirmations": 1
```

The wallet built the transaction (one input, a 1.0 BTC payment output and a
change output on the internal keychain), signed the P2WPKH input, broadcast it
via `sendrawtransaction`, and after `sync` the change was tracked and the balance
updated. A Taproot wallet (`init --taproot`) produces `5120...` outputs with
Schnorr witnesses through the same path.

## Project structure

```
src/
  main.rs      clap CLI, one function per subcommand, tracing setup
  config.rs    load + validate .env; `init` bootstrap that seeds the mnemonic
  error.rs     WalletError (thiserror) - one variant per failure class
  keys.rs      BIP39 mnemonic -> BIP84/BIP86 descriptor pair (wpkh | tr)
  wallet.rs    BDK wallet bound to its SQLite connection; create / open / query
  node.rs      bitcoincore-rpc client: connect + network check, broadcast,
               fee estimate, regtest mining
  sync.rs      bdk_bitcoind_rpc Emitter loop: blocks + mempool -> wallet -> persist
  tx.rs        parse a spend request, build + sign the PSBT, broadcast
  raw_demo.rs  raw rust-bitcoin: decode any tx, hand-build an OP_RETURN
tests/
  wallet.rs    unit tests (derivation, raw demo) + an ignored end-to-end test
scripts/
  regtest.sh   start/stop a local regtest bitcoind
```

### Descriptor structure and why

The wallet is defined by **two string descriptors** that share one extended
private key and differ only in the last derivation step:

```
wpkh(tprv.../84'/1'/0'/0/*)#checksum   external / receiving
wpkh(tprv.../84'/1'/0'/1/*)#checksum   internal / change
```

- This is BDK's native model: a `Wallet` *is* an (external, internal) descriptor
  pair. Keeping change on its own keychain is exactly what the assignment asks
  for, and BDK handles gap-limit scanning and change-address rotation from it.
- BIP84 (`m/84'`) for `wpkh`, BIP86 (`m/86'`) for `tr`; coin type `1'` because
  the wallet is testnet/regtest only (mainnet is rejected in config).
- The descriptors are **re-derived from the mnemonic on every run** and never
  written to disk by the program. Only the mnemonic (in `.env`) and the chain
  data (in SQLite) persist. On `open`, the derived descriptors are checked
  against what SQLite recorded, so a changed mnemonic is caught immediately.

## Which library where, and why

**BDK (`bdk_wallet`)** - everything that is "wallet policy":

- descriptor parsing and key derivation (`descriptor!` macro, `keys::bip39`)
- the external/internal keychain split and address revelation
- UTXO tracking, balance (confirmed / pending / immature), and the transaction
  graph
- coin selection (branch-and-bound by default) and automatic change output
- PSBT construction (`build_tx`) and signing (`wallet.sign`)
- persistence: the `rusqlite` `WalletPersister` writes the changeset to SQLite,
  so reopening is a load, not a rescan

Hand-rolling any of this (UTXO selection, change handling, gap limits) would be
error-prone and is squarely what BDK exists to do.

**`bitcoincore-rpc`** - everything that talks to the node:

- `connect`: `getblockchaininfo` for a reachability + network-match check
- `send_raw_transaction` to broadcast the signed transaction
- `getrawtransaction` to read the broadcast tx back as proof
- `estimatesmartfee` for a fee rate (with a 1 sat/vB fallback for fresh regtest)
- `generatetoaddress` for the regtest funding helper

**`bdk_bitcoind_rpc`** - the bridge. Its `Emitter` walks the chain from the
wallet's last checkpoint using only public RPCs and yields blocks + mempool
transactions that BDK folds in with `apply_block_connected_to` /
`apply_unconfirmed_txs`. It wraps `bitcoincore-rpc`, so the node integration is
still "via bitcoincore-rpc", just at a higher level than issuing `getblock`
calls by hand. Core can even run with `-disablewallet`.

**`rust-bitcoin`** - re-exported by BDK, used directly where there is no wallet
context: `Address` / `Amount` / `FeeRate` / `OutPoint` parsing in the CLI layer,
and `raw_demo.rs` (see below).

### A scenario for raw `rust-bitcoin` instead of BDK

BDK's `Wallet` only deals with transactions it owns and outputs that are
payments. Two common tasks fall outside that:

1. **Inspecting a third-party transaction** - e.g. you are handed a raw hex from
   a block explorer or a peer and need to see its inputs, outputs and script
   types. There is no wallet involved, so BDK gives you nothing; you
   `consensus::deserialize` it into a `bitcoin::Transaction` and read the fields.
2. **Building a non-payment output** - an `OP_RETURN` data carrier, or any custom
   script. `TxBuilder` is built around `add_recipient(script, amount)` for
   spendable outputs; a zero-value data output is assembled by hand from
   `TxOut { value, script_pubkey }`.

`src/raw_demo.rs` does both, and `rfbwallet raw-demo` runs it:

```rust
// decode an arbitrary transaction - no wallet
let tx: Transaction = deserialize_hex(hex)?;
for txout in &tx.output {
    println!("{} sat  {}", txout.value.to_sat(), describe_script(&txout.script_pubkey));
}

// hand-build an OP_RETURN output BDK will not build for you
let mut payload = PushBytesBuf::new();
payload.extend_from_slice(message.as_bytes())?;
let data_output = TxOut {
    value: Amount::ZERO,
    script_pubkey: ScriptBuf::new_op_return(&payload),
};
let tx = Transaction {
    version: Version::TWO,
    lock_time: LockTime::ZERO,
    input: vec![placeholder_input],
    output: vec![data_output],
};
```

## Testing

```bash
cargo test                       # unit tests, no node needed

# end-to-end against a running regtest node:
RFB_IT_RPC=127.0.0.1:18443 \
RFB_IT_COOKIE=/path/to/regtest/.cookie \
cargo test --test wallet -- --ignored --nocapture
```

The ignored test creates a wallet in a temp dir, mines to it, sends a payment,
drops the wallet, reopens it from SQLite and asserts the confirmed UTXOs are
still there.

## Known limitations / what I would improve with more time

- **Sync is a full block scan.** `bdk_bitcoind_rpc`'s `Emitter` fetches every
  block from `start_height`. Fine for regtest; on testnet you must set
  `START_HEIGHT` sensibly. A compact-block-filter (BIP158) or Electrum/Esplora
  chain source would be far lighter.
- **No reorg handling beyond what BDK does automatically.** The emitter and
  `apply_block_connected_to` cope with checkpoint rollbacks, but there is no
  user-facing reporting when it happens.
- **Fee estimation is basic** - one `estimatesmartfee` call, 1 sat/vB fallback,
  no RBF fee-bump command (transactions do signal RBF).
- **The mnemonic is stored in plaintext in `.env`.** Acceptable for test
  networks and required by the assignment's "no hardcoded keys" rule, but a real
  wallet needs an encrypted store or a hardware signer.
- **Single account, single signer.** No multisig, no passphrase rotation, no
  multiple accounts. The descriptor model would extend to these but the CLI does
  not.
- **`send` waits for the caller to mine/confirm.** There is no mempool
  monitoring; you run `sync` again after a block.
