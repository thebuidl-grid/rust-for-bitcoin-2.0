# My Regtest Bitcoin Wallet

This is my submission for the Week 6 assignment: a small command-line Bitcoin wallet, written in
Rust, that only ever talks to **regtest**. It generates its own keys from a fresh BIP39 seed,
derives a BIP84 (`wpkh`) descriptor with separate receiving and change keychains, tracks its UTXOs
and balance, keeps everything in a local SQLite file so I can close it and open it again later, and
can build, sign and broadcast a real transaction against my own Bitcoin Core node. It's built on
[`bdk_wallet`](https://docs.rs/bdk_wallet), [`rust-bitcoin`](https://docs.rs/bitcoin/0.32.102/bitcoin/index.html)
and [`bitcoincore-rpc`](https://docs.rs/bitcoincore-rpc). (The original assignment brief is still
sitting in [`Readme.md`](Readme.md), lowercase r, if you want to compare what was asked for against
what I actually did.)

I kept this deliberately small. There's no GUI, no server, no multisig, no Lightning — just enough
of a wallet to prove I can wire these libraries together correctly and explain every part of it.

## 1. What it does

One binary, `wallet`, six subcommands:

```
cargo run -- create-wallet
cargo run -- wallet-info
cargo run -- new-address [--internal]
cargo run -- sync
cargo run -- balance
cargo run -- send --to <ADDRESS> --amount <BTC>
```

The first time you run any of these, the wallet generates a fresh 12-word BIP39 seed and derives a
BIP84 descriptor pair from it — one branch for receiving, one for change. Every command after that
just opens the same local SQLite database and picks up exactly where it left off. No re-scanning
the chain from scratch, no re-typing a seed phrase.

## 2. The libraries, and why I reached for each one

| Library | What I used it for | Why |
|---|---|---|
| **`bdk_wallet`** | Descriptors, keychains, address derivation, UTXO tracking, balance, coin selection, PSBT signing | This is the actual wallet engine, and honestly the whole point of using it. Descriptor parsing, BIP32/BIP84 derivation, a transaction graph for tracking UTXOs, coin selection, PSBT signing — getting any of that wrong by hand would be a real bug in a real wallet, so [src/wallet.rs](src/wallet.rs) is deliberately a thin layer on top of BDK, not a reimplementation of it. |
| **`bdk_bitcoind_rpc`** | Syncing the wallet from my Bitcoin Core node | This is BDK's supported way to turn a plain Bitcoin Core node into a chain source, without standing up Electrum or Esplora just for a lab assignment. Its `Emitter` walks blocks from the node and reports mempool contents, and the wallet applies both to keep its UTXO set current. |
| **`bitcoincore-rpc`** | Connecting to the node, checking chain state, broadcasting the signed transaction | The wallet never touches Bitcoin Core's own wallet — it treats the node purely as a data source and a broadcast pipe. `wallet-info` calls `getblockchaininfo`; `send` calls `sendrawtransaction` on the transaction BDK just built and signed. |
| **`bitcoin` (rust-bitcoin)** | `Xpriv`, `Address`, `Amount`, `FeeRate`, `Network`, `Txid` | Pinned to the same 0.32.x line that `bdk_wallet` and `bitcoincore-rpc` are built against, so the same `Transaction`/`Address`/etc. types pass between all three crates with no conversion glue anywhere. §11 walks through the one spot where I use this directly instead of going through BDK. |
| **SQLite, via `bdk_wallet`'s built-in `rusqlite` persister** | Local persistence | One file holds everything: BDK's own tables (descriptors, address indices, transaction graph, UTXOs), plus one small table I added myself for the wallet's seed phrase. More on this in §10. |
| **`clap`** | The CLI | Six subcommands with flags is exactly what `clap`'s derive macros are for — nicer than hand-rolling `std::env::args()` parsing. |
| **`dotenvy`** | Loading `.env` | Keeps my RPC credentials out of the source tree and out of shell history. |

## 3. How it fits together

```
CLI (clap, src/main.rs)
        |
        v
Wallet logic (src/wallet.rs)
        |
   +----+----------------------------+
   |                                 |
   v                                 v
bdk_wallet                    bdk_bitcoind_rpc
(descriptors, keychains,      (Emitter: blocks + mempool
 UTXOs, PSBT signing)          from the node)
   |                                 |
   +----------------+----------------+
                    v
            bitcoincore-rpc
                    |
                    v
          Bitcoin Core (regtest)

SQLite (src/db.rs), sitting alongside the above:
  - bdk_wallet's own tables: descriptors, chain state, tx graph, UTXOs
  - one extra table I added: the wallet's BIP39 seed phrase
```

What's actually in each file:

| File | What it's responsible for |
|---|---|
| [src/main.rs](src/main.rs) | The CLI itself (`clap`) and dispatching to the right command. Loads `.env`, opens the database, opens the wallet, prints results. |
| [src/wallet.rs](src/wallet.rs) | All the wallet logic: seed → descriptors, create/load, sync, new address, balance, build/sign/broadcast. This is the file I'd point to first if you asked "where's the actual wallet." |
| [src/rpc.rs](src/rpc.rs) | Builds a `bitcoincore_rpc::Client` from `.env`. |
| [src/db.rs](src/db.rs) | Opens the SQLite file and owns the one extra `wallet_seed` table. |
| [src/error.rs](src/error.rs) | One `WalletError` enum, used everywhere, so every command hands back a `Result` instead of panicking on bad input. |

## 4. What you'll need

- Rust (stable) and Cargo
- A Bitcoin Core node running in regtest mode (I built and tested this against v27)

## 5. Setting up Bitcoin Core

You need a `bitcoind` running in regtest mode somewhere. If you've already got one — Polar counts —
just point `.env` at it and skip ahead to §6. If you're starting from nothing:

```bash
# Grab Bitcoin Core (this is the Linux x86_64 build; see bitcoincore.org for other platforms)
curl -O https://bitcoincore.org/bin/bitcoin-core-27.0/bitcoin-27.0-x86_64-linux-gnu.tar.gz
tar xzf bitcoin-27.0-x86_64-linux-gnu.tar.gz

# Start it in regtest mode with RPC auth. -daemon backgrounds it.
mkdir -p ~/bitcoin-regtest-data
./bitcoin-27.0/bin/bitcoind \
  -regtest -daemon \
  -datadir=~/bitcoin-regtest-data \
  -rpcuser=rfbuser -rpcpassword=rfbpass

# Quick sanity check that it's actually up
./bitcoin-27.0/bin/bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass getblockchaininfo
```

Worth noting: the wallet never opens or uses Bitcoin Core's *own* wallet — it only ever calls
node-level RPCs (`getblockchaininfo`, block/mempool reads through `bdk_bitcoind_rpc`,
`sendrawtransaction`). So you don't need to create or load a Core wallet for any of this to work —
a bare node is enough.

## 6. Environment

Copy [`.env.example`](.env.example) to `.env` and fill in whatever RPC credentials you started
`bitcoind` with:

```bash
cp .env.example .env
```

```
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=changeme
BITCOIN_RPC_PASSWORD=changeme
NETWORK=regtest
DATABASE_URL=data/wallet.db
```

`.env` and `data/` (which is where the SQLite file lives, seed phrase and all) are both gitignored.
There's nothing in this repo — not in the code, not in git history — that contains a real key or
seed. Everything here was generated fresh, on regtest, for this assignment.

## 7. Running it

```bash
cargo run -- create-wallet          # generates a fresh seed + descriptors on first run
cargo run -- wallet-info            # shows descriptors + whether the node is reachable
cargo run -- new-address            # next receiving address
cargo run -- new-address --internal # next change address
cargo run -- sync                   # pulls blocks/mempool from Bitcoin Core, updates balance
cargo run -- balance
cargo run -- send --to bcrt1q... --amount 0.1
```

`create-wallet` is really just a convenience — every other command opens (and, the first time,
creates) the wallet on its own, so you never technically have to run it. I kept it as an explicit
command anyway since it made the demo below easier to narrate.

### How to read the `balance` output

`balance` prints up to five lines, and they should always add up:

```
Confirmed balance: 1.00000000 BTC
Pending balance:   0.10000000 BTC (unconfirmed)
Immature balance:  50.00000000 BTC (coinbase reward(s), not yet spendable)
Total balance:     51.10000000 BTC
Spendable now:     1.10000000 BTC
```

- **Confirmed** + **Pending** + **Immature** = **Total**, always. `Pending` and `Immature` only
  print when they're non-zero, so a normal wallet with nothing pending or unmatured just shows
  `Confirmed` and `Total` — equal to each other, matching the simple example in the assignment
  brief.
- **Immature** is coinbase money — from blocks this wallet itself mined — that hasn't hit the
  100-confirmation maturity rule yet. It's real money, just not spendable *yet*. If you fund a
  fresh regtest wallet by mining to its own address, almost all of that balance will sit in
  `Immature` until 100 more blocks go by.
- **Spendable now** is the number that actually matters before you run `send` — confirmed coins,
  plus unconfirmed change from your *own* transactions. Money someone else just sent you doesn't
  count until it confirms, on purpose, so an unconfirmed incoming payment can't be spent and then
  vanish if it never confirms.

I added that last section (and the `Immature`/`Spendable now` lines) after catching a bug in my own
first draft — see §12 for the story, it's a decent example of BDK doing the right thing while my
own display code was misleading.

## 8. Walking through a real transaction

This is the actual sequence I ran to prove the wallet end-to-end (I've shortened `cargo run --` to
just `wallet` below, as if using a release build):

```bash
# 1. Create the wallet and grab a receiving address
wallet create-wallet
wallet new-address
# -> bcrt1qwsyj89evlheqwcjul0unymx8ug7mxlq8rsy6js

# 2. Fund it: mine 101 regtest blocks to that address (need 100 confirmations to mature a coinbase)
bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  generatetoaddress 101 bcrt1qwsyj89evlheqwcjul0unymx8ug7mxlq8rsy6js

# 3. Sync and check the balance
wallet sync
wallet balance
# -> Confirmed balance: 100 BTC   (2 matured 50 BTC coinbases; the rest still immature)

# 4. Spin up a second wallet (just a different local DB) to receive the payment
DATABASE_URL=data/wallet_2.db wallet create-wallet
DATABASE_URL=data/wallet_2.db wallet new-address
# -> bcrt1qv6cwprj0ezekhtpm03xfgw5v3297l8h37lq0an

# 5. Send from wallet 1 to wallet 2
wallet send --to bcrt1qv6cwprj0ezekhtpm03xfgw5v3297l8h37lq0an --amount 1.5
# Transaction created.
# Transaction signed.
# Transaction broadcast.
#
# TXID: cc34596198c5adcf09614ae1426e4503f01f16bb949172b74bbf15b4f785f770

# 6. Double-check it independently, straight from Bitcoin Core, not just trusting my own wallet
bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  getrawtransaction cc34596198c5adcf09614ae1426e4503f01f16bb949172b74bbf15b4f785f770 true
# vout shows: 1.5 BTC -> bcrt1qv6cwprj0... (the recipient)
#             48.499... BTC -> bcrt1qamhuj0... (change, back to wallet 1's INTERNAL keychain)

# 7. Mine a confirmation and sync both wallets
bitcoin-cli -regtest -rpcuser=rfbuser -rpcpassword=rfbpass \
  generatetoaddress 1 bcrt1qwsyj89evlheqwcjul0unymx8ug7mxlq8rsy6js
wallet sync
DATABASE_URL=data/wallet_2.db wallet sync
DATABASE_URL=data/wallet_2.db wallet balance
# -> Confirmed balance: 1.50000000 BTC
```

That txid is real — it's from an actual run of this wallet against my own regtest node, not made up
for the README.

## 9. Descriptors, explained the way I'd explain them to a friend

A **descriptor** is just a string that tells the wallet exactly how to compute every address it
owns — the script type, the key(s) behind it, and the derivation path — instead of the wallet
keeping a flat list of addresses somewhere. This wallet has two:

```
wpkh([fcb2e48e/84'/1'/0']tpub.../0/*)   <- external / receiving
wpkh([fcb2e48e/84'/1'/0']tpub.../1/*)   <- internal / change
```

- **`wpkh(...)`** — "pay to witness public key hash": a native SegWit address, the `bcrt1q...`
  ones. This is what BIP84 standardizes.
- **`[fcb2e48e/84'/1'/0']`** — where this key came from: `fcb2e48e` is the master key's
  fingerprint, and the rest is the hardened derivation path used to get here — purpose `84'`
  (BIP84), coin type `1'` (testnet/regtest, per BIP44), account `0'`.
- **`tpub...`** — the extended *public* key at that path. Addresses are computed from this alone;
  you only need the matching private key to actually *sign* something, not to watch the wallet or
  generate new addresses.
- **`/0/*`** vs **`/1/*`** — the two keychains. `0` is **external** — hand these out to get paid.
  `1` is **internal** — the wallet uses these on its own for leftover change, and they should never
  be given to anyone. The `*` just means "derive a fresh one on demand", one per address requested
  (index 0, then 1, then 2, and so on).

Keeping receiving and change on two separate branches like this is exactly why a block explorer (or
a watch-only Core wallet) can tell "this address was handed out to get paid" apart from "this is the
sender quietly getting their own leftover money back" — just by which branch the address sits on.

## 10. How persistence actually works here

Everything lives in one SQLite file (`DATABASE_URL`, `data/wallet.db` by default):

- **`bdk_wallet`'s own tables**, created and managed entirely by the crate itself through its
  built-in `rusqlite` persister — the wallet's chain state: which addresses have been revealed on
  each keychain, the transaction graph, and which outputs are still unspent. Anything that changes
  this (`new-address`, `sync`, `send`) calls `wallet.persist(&mut conn)` before the command exits.
- **`wallet_seed`**, a table I added myself in [src/db.rs](src/db.rs) — one row, holding the
  wallet's BIP39 mnemonic. This is genuinely the only secret this wallet has. Every time the binary
  runs, [src/wallet.rs](src/wallet.rs) reads that row back, re-derives the same master key and the
  same two descriptors from it (private keys included), and hands them to BDK so it can sign — while
  BDK's own tables only ever end up storing the *public* half of those descriptors.

So "restart the wallet" is just "run the binary again against the same file": no re-typing a seed,
no re-scanning from genesis (sync just resumes from the last checkpoint it already had).

## 11. Where I reached for `rust-bitcoin` directly instead of going through BDK

Most of this wallet goes through `bdk_wallet` on purpose — see §2 for why. There's exactly one spot
where I drop down to `rust-bitcoin` directly: turning the wallet's BIP39 seed into a master extended
private key, in [`xprv_from_mnemonic`](src/wallet.rs):

```rust
use bdk_wallet::bitcoin::bip32::Xpriv;

fn xprv_from_mnemonic(mnemonic: &Mnemonic, network: Network) -> Result<Xpriv> {
    let seed = mnemonic.to_seed(""); // BIP39: mnemonic + passphrase -> 64-byte seed
    Xpriv::new_master(network, &seed).map_err(WalletError::app)
}
```

BDK's descriptor templates (`bdk_wallet::template::Bip84`) expect a key to derive *from* — not a raw
seed — so turning the seed into a master key (`Xpriv::new_master`, plain BIP32) has to happen before
BDK enters the picture at all. That's a genuinely natural boundary, not an arbitrary one: BIP32/BIP39
key material is a `rust-bitcoin` concept on its own, independent of any particular wallet library;
BDK's job starts the moment you hand it a key plus a derivation scheme. Doing this step in
`rust-bitcoin` directly, rather than trying to coax BDK into generating its own root key, keeps that
line visible in the code — you can point at exactly where "plain Bitcoin cryptography" stops and
"wallet policy" starts.

## 12. A bug I actually found (and fixed) while building this

Worth writing down, since it was a good lesson: my first version of `balance` printed `Confirmed`,
`Pending`, and `Total` — but `Total` is BDK's `.total()`, which also folds in a fourth number,
`immature` (unmatured coinbase), that I just wasn't printing. So the three numbers I did show didn't
add up:

```
Confirmed balance: 100 BTC
Pending balance:   48.49999719 BTC
Total balance:     3448.49999719 BTC     <- where did the rest of this come from?
```

That looked exactly like a broken balance calculation. It wasn't — BDK's math was correct the whole
time, I'd just never surfaced where the other ~3300 BTC was actually sitting (unmatured coinbase
from blocks I'd mined for testing). I fixed the display, not the math: added an `Immature` line so
every component is visible and the numbers reconcile, plus a `Spendable now` line so it's obvious
what you can actually send without doing the arithmetic yourself. See §7 for what it looks like now.

## 13. Known limitations

- **Regtest only.** I haven't tested anything else, and the CLI prints a warning if `NETWORK` is set
  to anything but `regtest`.
- **One wallet per database file.** You get multiple wallets by pointing `DATABASE_URL` at different
  files (like I did in the demo above), not by one file holding several wallets at once.
- **Fixed fee rate.** `send` always uses a hardcoded 2 sat/vB instead of asking Bitcoin Core for a
  fee estimate — regtest doesn't have a real mempool to estimate from anyway, and a fixed rate keeps
  the demo simple and repeatable.
- **Default coin selection.** `send` uses BDK's default algorithm rather than me picking UTXOs by
  hand.
- **The seed sits in plaintext** in the local SQLite file — allowed for a local test wallet under
  this assignment's constraints (see [`Readme.md`](Readme.md)), but a real wallet would encrypt this
  at rest. I'd do that next if I kept building on this.
- **No RBF/fee-bumping, no watch-only mode, no hardware wallet support, no multisig, no Lightning.**
- **CLI only** — no GUI, no long-running daemon.

## 14. Stretch goals I attempted

- **A CLI** (`clap`) — all six commands above.
- **Basic error handling** — one `WalletError` type ([src/error.rs](src/error.rs)); no
  `.unwrap()` anywhere in the application logic, every command returns a `Result` and prints a
  message instead of panicking (I actually tested this — insufficient funds, a wrong-network
  address, bad RPC credentials — all fail cleanly).
- **Using `rust-bitcoin` directly instead of BDK** somewhere real — see §11.

I left coin selection at BDK's default rather than making it explicit — see §13 for why, mostly a
time-tradeoff for this assignment rather than a hard limitation.
