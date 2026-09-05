# RFB Week 6 — A Regtest/Testnet Bitcoin Wallet in Rust

A CLI wallet built on [`bdk_wallet`](https://docs.rs/bdk_wallet) (descriptors, keychains,
coin selection, PSBT signing, SQLite persistence) and
[`bitcoincore-rpc`](https://docs.rs/bitcoincore-rpc) / [`bdk_bitcoind_rpc`](https://docs.rs/bdk_bitcoind_rpc)
(chain sync and broadcast against a real Bitcoin Core node). No private key material is
ever hardcoded — the BIP39 mnemonic lives only in a local, gitignored `.env` file.

## 1. Setup

### 1a. Get a Bitcoin Core node running (regtest)

The wallet needs a real node to talk to. Easiest path is Docker:

```bash
docker run -d --name rfb-week6-regtest \
  -p 18443:18443 -p 18444:18444 \
  ruimarinho/bitcoin-core:24 \
  -regtest=1 -server=1 \
  -rpcallowip=0.0.0.0/0 -rpcbind=0.0.0.0 \
  -rpcuser=rfbuser -rpcpassword=rfbpass \
  -fallbackfee=0.0002 -txindex=1
```
d
Then create a second, node-side "miner" wallet you'll use only to fund this wallet's
addresses in testing (our wallet never touches this one):

```bash
docker exec rfb-week6-regtest bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  createwallet miner
MINER_ADDR=$(docker exec rfb-week6-regtest bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  -rpcwallet=miner getnewaddress)
docker exec rfb-week6-regtest bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  generatetoaddress 101 "$MINER_ADDR"   # matures a coinbase output to spend from
```

(On real testnet, skip all of the above and point `RPC_URL`/`RPC_USER`/`RPC_PASS` at a
node you already have — e.g. a public testnet Electrum-backed node isn't enough, you
need actual Bitcoin Core JSON-RPC, per the assignment's constraint.)

### 1b. Configure the wallet

Copy `.env.example` to `.env` (already gitignored) and adjust if your node's host, port,
or credentials differ from the defaults above:

```bash
cp .env.example .env
```

`.env` fields:

| Variable | Default | Meaning |
|---|---|---|
| `BITCOIN_NETWORK` | `regtest` | `bitcoin` / `testnet` / `signet` / `regtest` |
| `RPC_URL` | `127.0.0.1:18443` | Bitcoin Core RPC host:port |
| `RPC_USER` / `RPC_PASS` | `rfbuser` / `rfbpass` | RPC credentials (or set `RPC_COOKIE` to a cookie file path instead) |
| `WALLET_DB` | `wallet.sqlite` | Local SQLite state file |
| `DESCRIPTOR_KIND` | `wpkh` | `wpkh` (BIP84 native SegWit) or `tr` (BIP86 Taproot) |
| `ACCOUNT` | `0` | BIP32 account index |
| `MNEMONIC` | *(unset)* | Set by `init` the first time you run it — **never commit this** |
| `PASSPHRASE` | *(empty)* | Optional BIP39 passphrase |

Every field is also available as a `--flag` (e.g. `--network`, `--rpc-user`), which
overrides the `.env`/environment value — run `cargo run -- --help` or
`cargo run -- send --help` for the full list.

### 1c. Initialize the wallet

```bash
cargo run -- init
```

On first run this generates a fresh 12-word BIP39 mnemonic, appends
`MNEMONIC="..."` to `.env`, derives the external/internal BIP84 descriptors from it,
creates `wallet.sqlite`, and prints the first receive address. Re-running `init` later
just reopens the existing wallet (it refuses to silently overwrite an existing
`MNEMONIC` — remove it from `.env` yourself first if you really mean to start over).

## 2. Commands

```bash
cargo run -- init                        # create/open the wallet, print an address
cargo run -- address                     # reveal the next receive (external) address
cargo run -- change-address              # reveal the next change (internal) address
cargo run -- sync                        # sync against the node, report new blocks/tip
cargo run -- balance                     # sync, then print confirmed/pending/immature/total
cargo run -- utxos                       # sync, then list every known UTXO
cargo run -- send <address> <amount_sats>  # sync, build+sign+broadcast, print the txid
```

`balance`/`utxos`/`send` all sync first, so they always reflect current chain state —
you never have to remember to `sync` separately, though the standalone command is
there too.

### End-to-end example (regtest)

```bash
cargo run -- init
# First receive address: bcrt1q...

# fund it from the node-side miner wallet set up in 1a, then mine a confirmation:
docker exec rfb-week6-regtest bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  -rpcwallet=miner sendtoaddress bcrt1q... 1.5
docker exec rfb-week6-regtest bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  -rpcwallet=miner generatetoaddress 1 "$MINER_ADDR"

cargo run -- balance
# Confirmed: 1.5 BTC, Total: 1.5 BTC

cargo run -- send bcrt1q<some-other-address> 10000
# Broadcast transaction <txid>
```

See `PROOF.md` for a real run of this sequence against a live regtest node, with actual
addresses, balances, and a broadcast, block-confirmed txid.

## 3. Project structure & why

```
src/
├── main.rs         CLI entry point: load .env, parse args, dispatch to commands
├── config.rs        Cli/Config/Command — clap-derived, every field env-backed
├── mnemonic.rs      Generate a BIP39 mnemonic; append it to .env exactly once
├── descriptors.rs   Mnemonic -> BIP84/86 external+internal descriptors -> Wallet
├── sync.rs          Bitcoin Core RPC client + block/mempool sync via bdk_bitcoind_rpc
└── commands.rs       One function per subcommand (init/address/balance/utxos/send/...)
```

**Descriptor structure.** The wallet is two matched output descriptors derived from one
mnemonic: an *external* keychain at `m/84'/{coin}'/{account}'/0/*` for addresses you
hand out to be paid, and an *internal* keychain at `.../1/*` for the wallet's own
change. Keeping them as separate BDK `KeychainKind`s (rather than one descriptor doing
double duty) is what BIP44/49/84 wallets do and what `bdk_wallet` is built around — it's
what lets `reveal_next_address` hand out receive addresses without ever reusing them,
while change never shows up as something a human is expected to reuse or verify.
`DESCRIPTOR_KIND=tr` swaps the same two-keychain shape onto BIP86 (`m/86'/...`) Taproot
descriptors instead, to compare script types without changing anything else about the
wallet (see §5).

**Persistence.** `bdk_wallet`'s `rusqlite` feature backs the wallet's `ChangeSet`
(keychain indices, the local chain of block checkpoints, and the transaction graph) with
a `Connection` to `wallet.sqlite`. `Wallet::load()` reconstructs a `PersistedWallet` from
that file on every run; if it's the first run, `Wallet::create()` initializes it instead.
Every state-changing operation (`reveal_next_address`, applying a synced block, applying
mempool transactions, signing/broadcasting a send) is immediately followed by
`wallet.persist(&mut db)`, so closing and reopening the wallet — including switching
which physical machine you run it on — never loses track of what's already synced or
which addresses have been handed out.

## 4. Which library did what, and why

| Library | Used for | Why this one |
|---|---|---|
| `bdk_wallet` | Descriptor parsing, keychain-aware address derivation, coin selection (`build_tx`), PSBT construction, signing (`wallet.sign`), and the SQLite-backed `ChangeSet`/`PersistedWallet` persistence model | It's the layer above raw `rust-bitcoin` that actually tracks "which scriptPubKeys are mine, which are spent, what's my balance" — reimplementing that state machine by hand over raw `rust-bitcoin` would be most of a wallet's real complexity for no benefit here |
| `bdk_bitcoind_rpc` (`Emitter`) | Walking new blocks since the wallet's last checkpoint and pulling the current mempool, in a form `bdk_wallet::Wallet::apply_block_connected_to`/`apply_unconfirmed_txs` can consume directly | This is the glue BDK ships specifically for a Bitcoin Core RPC backend — it turns `getblock`/`getrawmempool` polling into the checkpoint-aware update events the wallet expects, including reorg handling via `last_cp` |
| `bitcoincore-rpc` (directly, via `RpcApi`) | `send_raw_transaction` to broadcast the signed tx, and a `getblockchaininfo` probe on startup so a bad RPC URL/credentials fails immediately with a clear error instead of hanging inside the first sync | `bdk_wallet` deliberately does not broadcast — building/signing a PSBT is its job, actually getting the bytes to a node is the caller's, so this is the one place raw `bitcoincore-rpc` calls are made directly rather than through BDK |
| `bip39` / `miniscript` (via `bdk_wallet`'s `keys-bip39` feature) | Generating/parsing the 12-word mnemonic and building `descriptor!(wpkh(...))`/`descriptor!(tr(...))` output descriptors with embedded key-origin info from it | Re-exported by `bdk_wallet` specifically so the mnemonic-to-descriptor path (`examples/mnemonic_to_descriptors.rs` in the crate) is exercised the way the library intends, rather than hand-rolling BIP32 derivation and descriptor string formatting |
| `clap` (derive + `env`) | The whole CLI surface — subcommands, and every config field readable from either a flag or an environment variable | `env = "..."` on each field is what makes `.env` values and `--flag` overrides compose for free, with `--help` documenting both |
| `dotenvy` | Loading `.env` into the process environment before `clap` parses it | Standard, small, does exactly the one thing needed here |
| `anyhow` | Error propagation/context across all of the above (`bdk_wallet`, `bitcoincore-rpc`, `rusqlite`, and I/O each have their own error types) | `.context("...")` at each fallible step turns a bare RPC/DB error into an actual explanation of what the wallet was trying to do, without a hand-written error enum for every library boundary |

Raw `rust-bitcoin` types (`Address`, `Amount`, `Transaction`, `DerivationPath`, ...) are
used throughout, but always via `bdk_wallet::bitcoin::*` re-exports rather than a direct
dependency, specifically so the whole crate graph is guaranteed to agree on one
`rust-bitcoin` version — this project never adds `bitcoin` itself as a dependency.

## 5. Stretch goals attempted

- **`wpkh` vs `tr` descriptors** (`DESCRIPTOR_KIND=wpkh|tr` / `--descriptor-kind`): the
  exact same wallet code path derives a BIP86 Taproot wallet instead of BIP84 native
  SegWit — only the `purpose'` level of the derivation path and which `descriptor!`
  macro arm gets called differ (`src/descriptors.rs`). `cargo run -- --descriptor-kind tr init`
  produces `bcrt1p...` addresses from the same mnemonic on a separate, independent
  `wallet.sqlite` (`WALLET_DB` should be changed too, since a `wpkh` and `tr` wallet from
  the same mnemonic/account are different coins as far as the chain is concerned).
- **A CLI usable without editing code**: every operation (`init`, `address`,
  `change-address`, `sync`, `balance`, `utxos`, `send`) is a subcommand with `--help`,
  driven entirely by flags/env vars — nothing here requires touching the source to
  check a balance or send funds.
- **Basic error handling throughout**: every fallible RPC/DB/signing/parsing step is
  wrapped in `anyhow::Context` with a specific message (see `src/commands.rs`,
  `src/sync.rs`); a bad address, insufficient funds, an unreachable node, or a
  partially-signed PSBT all produce a readable one-line error and a non-zero exit
  instead of a panic.

Coin selection is left at BDK's default largest-first algorithm rather than made
explicit — see Limitations.

## 6. Known limitations / what I'd improve with more time

- **Coin selection is BDK's default**, not hand-rolled. Explicit control (e.g. picking a
  specific `OutPoint` set, or a privacy-motivated selection strategy) would go through
  `TxBuilder::add_utxos`/`manually_selected_only`, which this CLI doesn't expose yet.
- **No fee-rate control on `send`** — it relies on `bdk_wallet`'s default fee estimation
  path, which on regtest with no mempool history effectively falls back to a fixed
  rate. A real `--fee-rate` flag (feeding `TxBuilder::fee_rate`) would matter a lot on
  testnet.
- **Single-descriptor wallets only.** No multisig, no descriptor-per-UTXO mixing; adding
  a `sh(wsh(multi(...)))` template alongside `wpkh`/`tr` would follow the same shape as
  §5's stretch goal.
- **No automatic block-tip following.** `sync` is one-shot; a long-running `watch`
  subcommand that re-polls on an interval (rather than requiring a fresh `cargo run --
  sync`/`balance` each time) would make it feel more like a real wallet daemon.
- **`.env`-based key storage is intentionally the simplest thing that satisfies the
  assignment's "don't hardcode keys" constraint**, not a real key-management story — a
  production version would want an OS keychain or an encrypted-at-rest store instead of
  a plaintext file, gitignored or not.
