# rfbwallet — a descriptor-based Bitcoin wallet in Rust

Week 6 assignment. A working wallet on regtest that generates keys from a seed,
derives addresses from separate receive and change keychains, tracks its own UTXOs
against a Bitcoin Core node, persists everything to SQLite, and builds, signs and
broadcasts transactions.

Assignment brief: [`ASSIGNMENT.md`](./ASSIGNMENT.md) · Build plan and working
notes: [`PLAN.md`](./PLAN.md)

```
$ rfbwallet send --to bcrt1q6tqyfukzez7fusm5pczmerwqczzq4z4z3ezyag --amount 250000000

  paying        2.50000000 BTC  (   250000000 sat) to bcrt1q6tqyfukzez7...
  fee rate  2 sat/vB
  selection branch-and-bound

  txid      2deedc5213558f04a82d882c36d9c57d41922509db261e54064a4fb52b3fb164
  inputs    1
  outputs   2
  fee           0.00000281 BTC  (         281 sat)
  change    vout 1,    47.49999719 BTC  (  4749999719 sat) -> internal keychain
  size      141 vB

  broadcast ok
```

---

## Requirements coverage

| # | Requirement | Where |
|---|---|---|
| 1 | Generate/import keys, derive a wallet from a descriptor | [`keys.rs`](src/keys.rs) |
| 2 | Addresses from external **and** internal keychains | [`wallet.rs`](src/wallet.rs) |
| 3 | Track UTXOs and calculate balance | [`node.rs`](src/node.rs), [`wallet.rs`](src/wallet.rs) |
| 4 | Persist state locally (SQLite) | [`wallet.rs`](src/wallet.rs) |
| 5 | Construct, sign and broadcast a transaction | [`tx.rs`](src/tx.rs) |
| 6 | Connect to a Bitcoin node over RPC | [`node.rs`](src/node.rs) |

Stretch goals attempted: CLI, `wpkh` vs `tr` descriptors, explicit coin selection,
error handling, and a note on where raw `rust-bitcoin` is used instead of BDK.

---

## Setup

### 1. A regtest node

Developed against [Polar](https://lightningpolar.com/) running Bitcoin Core 30.0.
Create a network with **one bitcoind node and zero Lightning nodes**, then start it.

Polar is convenient here because of the flags it passes bitcoind:

```
-fallbackfee=0.0002    regtest has no fee history, so estimatesmartfee returns
                       nothing; without a fallback, transaction building fails
-txindex=1             full transaction index
-blockfilterindex=1    enables the BIP158 sync path (not used; see Limitations)
-rpcbind=0.0.0.0 -rpcallowip=0.0.0.0/0 -rpcport=18443
```

Any bitcoind works, as long as `-fallbackfee` is set. Polar's default credentials
are `polaruser` / `polarpass`, published and identical in every Polar network, so
they are not secrets.

Check it is reachable:

```bash
bitcoin-cli -regtest -rpcconnect=127.0.0.1 -rpcport=18443 \
  -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo
```

### 2. Build

```bash
cd rfb_labs_week_6
cargo build
```

Rust 1.85+ (edition 2024, BDK's MSRV). Developed on 1.96.

### 3. Create a wallet

```bash
cargo run -- init
```

Generates a 12-word BIP39 seed (`--words 24` for more), writes `.env` with mode
`0600`, and prints the phrase once. `.env` is gitignored. Running `init` again
refuses to overwrite an existing seed without `--force`.

Configuration lives in `.env` — see [`.env.example`](.env.example):

| Key | Default | Notes |
|---|---|---|
| `BITCOIN_NETWORK` | `regtest` | `regtest`/`signet`/`testnet`/`testnet4`. **Mainnet is refused.** |
| `WALLET_MNEMONIC` | — | Written by `init`. Must be quoted; the value has spaces. |
| `WALLET_PASSPHRASE` | empty | Optional BIP39 passphrase. |
| `WALLET_DB` | `./wallet.sqlite` | Wallet state. |
| `DESCRIPTOR_KIND` | `wpkh` | `wpkh` (BIP84) or `tr` (BIP86 Taproot). |
| `RPC_URL` / `RPC_USER` / `RPC_PASSWORD` | Polar defaults | Node connection. |

---

## Usage

```
init [--words 12|24] [--force]   generate a seed and write .env
info                             descriptors, fingerprint, network, db path
address new [--change] [--reveal]  show a receive or change address
address list                     every address revealed so far
sync                             pull blocks and mempool from the node
balance                          confirmed / immature / pending
utxos                            outpoint, value, keychain, index, status
fund [--blocks 101]              mine to this wallet (regtest only)
send --to <addr> --amount <sat> [--fee-rate <sat/vB>] [--largest-first] [--dry-run]
```

### End-to-end walkthrough

```bash
cargo run -- init
cargo run -- info
cargo run -- fund --blocks 101        # mine to ourselves; regtest, no faucet
cargo run -- balance
cargo run -- send --to <address> --amount 250000000 --dry-run
cargo run -- send --to <address> --amount 250000000
```

`fund` mines 101 blocks because **coinbase outputs need 100 confirmations**. A
coinbase at height `H` has `tip - H + 1` confirmations, so starting from tip 1 and
mining 101 blocks (tip 102) matures the coinbases at heights 2 and 3:

```
spendable    100.00000000 BTC  ( 10000000000 sat)   <- 2 mature coinbases
immature    4950.00000000 BTC  (495000000000 sat)   <- 99 still maturing
total       5050.00000000 BTC  (505000000000 sat)   <- 101 UTXOs
```

---

## Proof of a working transaction

Broadcast on regtest, then read back through the **node's** own view rather than
the wallet's:

```
$ bitcoin-cli ... getrawtransaction 2deedc5213558f04a82d882c36d9c57d41922509db261e54064a4fb52b3fb164 true

  txid          2deedc5213558f04a82d882c36d9c57d41922509db261e54064a4fb52b3fb164
  blockhash     0692c8ca44df56b215d5961aa95e546e184181a82cd2967ab556012b62b551ec
  confirmations 1
  vsize         141 vB
  vout 0      2.50000000 BTC  ->  bcrt1q6tqyfukzez7fusm5pczmerwqczzq4z4z3ezyag
  vout 1     47.49999719 BTC  ->  bcrt1qkqv3v678svq432rc07k72n6csn3n08fd0dzfdc
```

`vout 1` is change, and the wallet places it on the **internal** keychain:

```
$ rfbwallet utxos
  2deedc5213558f04a82d882c36d9c57d41922509db261e54064a4fb52b3fb164:1
       47.49999719 BTC  (  4749999719 sat)   internal  index 0    confirmed @ 103
```

Fee: 281 sat for 141 vB at 2 sat/vB.

---

## Project structure

```
src/
├── main.rs      35 lines   parse args, dispatch, print errors, exit code
├── cli.rs      728 lines   clap definitions + command implementations
├── config.rs   248 lines   .env into a validated Config; refuses mainnet
├── error.rs    149 lines   one error type for the crate
├── keys.rs     257 lines   mnemonic -> extended key -> xprv -> descriptors
├── wallet.rs   299 lines   load-or-create PersistedWallet; addresses, balance
├── node.rs     161 lines   RPC client + Emitter sync loop
└── tx.rs       190 lines   build, sign, broadcast
```

Built as a **library plus a binary**. Integration tests can only import from a lib
target, and the persistence test needs to construct a wallet, drop it, and reopen
it — that has to be real library code, not `main.rs` internals.

Only `cli.rs` and `main.rs` print. Everything below returns `Result<T>`, which is
what lets the tests drive the same code paths the binary does.

## Descriptor structure

The wallet is defined entirely by two descriptor strings:

```
wpkh([dbd66f9f/84'/1'/0']tpubDC6pm1sxUWCWp...zbQ1M/0/*)#39jmlcdu   external
wpkh([dbd66f9f/84'/1'/0']tpubDC6pm1sxUWCWp...zbQ1M/1/*)#q3h6zday   internal
└─┬─┘ └───┬───┘└───┬────┘└──────┬─────────┘ └┬┘ └┬┘
  │       │        │            │            │   └── wildcard: address index
  │       │        │            │            └────── keychain: 0 receive, 1 change
  │       │        │            └─────────────────── account xpub
  │       │        └──────────────────────────────── derivation path taken
  │       └───────────────────────────────────────── master fingerprint
  └───────────────────────────────────────────────── script type
```

Derivation chain: entropy → BIP39 mnemonic (+ optional passphrase) → PBKDF2 seed →
BIP32 master xprv → `m/84'/1'/0'` account → `/0/*` and `/1/*`.

**Why `wpkh` (BIP84) by default.** Native segwit is the common default, has the
cheapest single-sig spends of the widely supported types, and signs simply.
`DESCRIPTOR_KIND=tr` switches the whole wallet to BIP86 Taproot (`m/86'/1'/0'`,
`bcrt1p...` addresses) through one match arm in `keys.rs`. Same seed, same
fingerprint, different account — the two share no addresses.

**Why two keychains, not one.** Two independent reasons:

* **Privacy.** If change landed on the receive chain, an address you had published
  would be reused by your own change output, linking otherwise separate payments.
* **Recovery.** Seed restore scans forward until it sees N unused addresses in a
  row. Mixing change into that chain corrupts the count and can silently skip past
  funds.

---

## Libraries: what, where, and why

| Library | Version | Used for | Why this one |
|---|---|---|---|
| `bdk_wallet` | 3.1.0 | descriptor expansion, script-pubkey indexing, UTXO set, coin selection, PSBT signing, changeset persistence | This is the wallet state machine. Writing it by hand means reimplementing gap-limit tracking, canonicalisation and reorg handling — the parts that are easy to get subtly wrong. |
| `bdk_bitcoind_rpc` | 0.22.0 | pulling blocks and mempool from Core | Its `Emitter` handles reorgs and mempool eviction. It also re-exports a version-matched `bitcoincore-rpc`, which is how the crate keeps a single `bitcoin` in the graph. |
| `bitcoincore-rpc` | 0.19.0 | node connection, broadcast, `generatetoaddress` | Reached *through* `bdk_bitcoind_rpc`, never declared directly. |
| `rust-bitcoin` | 0.32.102 | `Xpriv`, `Address`, `Amount`, `FeeRate`, `Network`, `Psbt`, `Transaction` | Reached as `bdk_wallet::bitcoin`. Never declared directly. |
| `rusqlite` | 0.31.0 | SQLite persistence | Reached as `bdk_wallet::rusqlite`. Never declared directly. |
| `clap` | 4.6.6 | CLI | Derive API keeps argument definitions next to the commands. |
| `dotenvy` | 0.15.7 | `.env` loading | Keeps the seed out of the source tree. |
| `thiserror` | 2.0.20 | error type | Generates `Display`, `Error` and the `From` impls that make `?` work. |

### The dependency trap this project deliberately avoids

`bdk_wallet` and `bitcoincore-rpc` both depend on `rust-bitcoin`. If they resolve to
different major versions, every shared type — `Transaction`, `Txid`, `BlockHash` —
becomes a *distinct* type to the compiler, producing errors like
`expected bitcoin::Transaction, found bitcoin::Transaction`.

`Cargo.toml` therefore declares **neither `bitcoin` nor `rusqlite`**, and reaches
both through `bdk_wallet`'s re-exports. Verify at any time:

```bash
grep -c '^name = "bitcoin"$' Cargo.lock    # must be 1
grep -c '^name = "rusqlite"$' Cargo.lock   # must be 1
```

`Cargo.lock` is committed, because that single-version property depends on it.

### Where raw `rust-bitcoin` is used instead of BDK

BDK can go straight from a mnemonic to a wallet. This wallet deliberately stops at
the intermediate `Xpriv` (`keys.rs::master_xprv`) so it can report the master
fingerprint and account path in `info`, which BDK does not surface directly:

```rust
let key: (Mnemonic, Option<String>) = (mnemonic.clone(), passphrase.map(str::to_string));
let extended: ExtendedKey<Segwitv0> = key.into_extended_key()?;
let xprv = extended.into_xprv(network_kind).ok_or(WalletError::NoPrivateKey)?;

let secp = Secp256k1::new();
let fingerprint = xprv.fingerprint(&secp);   // raw rust-bitcoin
```

`Fingerprint` and `Secp256k1` are rust-bitcoin types with no BDK equivalent. The
same applies to `FeeRate` arithmetic in `tx.rs` and address network validation via
`require_network`.

### What Bitcoin Core is *not* used for

This wallet does not use Core's wallet at all — no `createwallet`, no
`importdescriptors`, no `getbalance`. Core does two jobs: **chain source** and
**broadcaster**. Key derivation, address generation, UTXO tracking, balance, coin
selection and signing all happen in this process through BDK. That is the point of
a descriptor wallet: the node never learns who you are.

---

## Design decisions

**Persistence is a changeset log, not a snapshot.** BDK stores deltas — descriptors,
last-revealed index per keychain, transaction graph, chain checkpoints. So *every
mutation must be followed by a persist*, and `reveal_next_address` /
`next_unused_address` / the transaction builder all flush before returning. Leaving
that to callers is exactly how you get silent address reuse.

**The database holds no private keys.** The `bdk_wallet` table stores the *public*
descriptor (`tpub...`); no `tprv` appears anywhere in the file. Losing
`wallet.sqlite` costs a resync, never the coins — the seed is the only
irreplaceable artifact. It is still gitignored and chmod'd `0600`, because it holds
the account xpub plus every address and transaction: a full privacy picture.

**`.extract_keys()` on load is required for spending**, for a reason that is easy to
misdiagnose. Signing keys come from the private descriptors passed to
`.descriptor(...)` at load time; without `extract_keys` BDK verifies against them
and then discards the secrets. Measured:

| load path | signing keys in key map |
|---|---|
| freshly created | 1 |
| reloaded **without** `extract_keys` | **0** |
| reloaded **with** `extract_keys` | 1 |

A wallet in that state syncs and reports balances perfectly, then fails to sign.

**`sign()` returns a bool that must be checked.** `Ok(false)` is not an error — it
means "no failure, but the PSBT is not fully signed", which is precisely what the
case above produces. Ignoring it broadcasts a transaction that can never confirm,
so it becomes `WalletError::IncompleteSignature` with a message naming the cause.

**`address new` defaults to `next_unused`, not `reveal_next`.** Asking repeatedly
returns the same unused address. If it always advanced, habitually checking your
address would march the index forward and a restore scanning ahead could walk past
funds. `--reveal` forces a fresh one.

**Mainnet is refused in code**, not just promised in prose. `rust-bitcoin` parses
`"bitcoin"` happily; `config.rs` rejects it explicitly.

**Amounts print in BTC and satoshis.** Fee mistakes are decimal-point mistakes.

---

## Coin selection

`--largest-first` swaps BDK's default `BranchAndBoundCoinSelection` for
`LargestFirstCoinSelection`. Measured on a 120 BTC payment from 101 identical
50 BTC coinbase UTXOs:

| algorithm | inputs | vsize | fee |
|---|---|---|---|
| branch-and-bound (default) | 3 | 276 vB | 553 sat |
| largest-first | 3 | 276 vB | 553 sat |

**Identical — an honest null result.** Branch-and-bound's advantage is finding a
combination that avoids a change output entirely, which is cheaper and better for
privacy. With 101 uniform UTXOs there is nothing to optimise: any 3 sum the same
and change is unavoidable. The algorithms would diverge on a mixed UTXO set. A
freshly mined regtest wallet is the worst possible place to demonstrate the
difference, and reporting a win the data does not show would be worse than
reporting this.

---

## Testing

```bash
cargo test
```

31 unit tests, no clippy warnings. The ones that carry real weight:

| Test | Asserts |
|---|---|
| `revealed_index_survives_restart` | reveal 2 → drop → reopen → index continues at 2, not 0 |
| `keychains_are_separate` | external `/0/*`, internal `/1/*`, addresses differ |
| `public_descriptors_carry_no_secret` | the shareable descriptor has `tpub` and never `tprv` |
| `written_env_round_trips_through_dotenvy` | `.env` parses back, not just "contains the text" |
| `rejects_mainnet` | mainnet is an error, not a panic |
| `script_kinds_produce_different_descriptors` | `wpkh(` vs `tr(`, `84'` vs `86'`, same fingerprint |

`revealed_index_survives_restart` was verified to actually fail when the `persist()`
call inside `reveal_next_address` is removed:

```
assertion `left == right` failed
  left: None            <- reopened wallet had revealed nothing
 right: Some(1)
```

Tests use the canonical BIP39 vector (`abandon abandon … about`), published in the
spec. No generated seed appears anywhere in the source.

---

## Known limitations

**No integration test against a live node.** Everything in `tests` runs offline. The
node-dependent paths — `connect`, `sync`, `broadcast` — were verified by hand and are
recorded in `PLAN.md`, but they are not automated. `bitcoind`-backed fixtures would
fix this and are the first thing I would add.

**Regtest only, in practice.** `signet` and `testnet` are accepted and should work,
but only regtest has been exercised. `fund` is regtest-only by design.

**Full block scan on every first sync.** `bdk_bitcoind_rpc` also offers `FilterIter`,
a BIP158 compact-filter path that fetches only blocks matching the wallet's scripts —
and Polar already enables `-blockfilterindex`. On a ~100-block regtest chain it buys
nothing and would cost mempool visibility, so `Emitter` is the right call here. On
testnet it would not be.

**No RBF or fee bumping.** A stuck transaction cannot be replaced. `TxBuilder`
supports `enable_rbf` and `build_fee_bump`; neither is wired up.

**No `send --max`.** `drain_to`/`drain_wallet` would give a sweep, which is the
natural way to empty a wallet without hand-computing the fee.

**Change output detection is a linear scan** over `tx.output` calling
`derivation_of_spk`. Fine for a handful of outputs; the wrong shape for a batched
payout.

**Single account, fixed at index 0.** `m/84'/1'/0'` is hardcoded. Multiple accounts
would need an `ACCOUNT` setting threaded through the templates.

**The `.env` quoting bug is worth recording.** `init` originally wrote
`WALLET_MNEMONIC=word word word…` unquoted. dotenv values end at the first space, so
every subsequent command failed with an opaque parse error. Five unit tests passed on
that broken file because they asserted the output *contained* the right substring
rather than that it *parsed*. The fix was one line; the lesson was that a test which
checks you wrote the bytes you intended is not a test of the contract.

---

## Attribution

Assignment by the Rust for Bitcoin 2.0 programme; brief preserved in
[`ASSIGNMENT.md`](./ASSIGNMENT.md). Written against
[BDK](https://bitcoindevkit.org/), [rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin)
and [rust-miniscript](https://github.com/rust-bitcoin/rust-miniscript).
