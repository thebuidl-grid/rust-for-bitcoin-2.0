# Week 6 assignment: a Bitcoin wallet in Rust

A descriptor wallet for regtest, built on `rust-bitcoin`, `bitcoincore-rpc` and BDK.
It generates or imports a BIP39 seed, derives external and internal keychains from a
descriptor, tracks its own UTXOs, keeps its state in SQLite, syncs from a Bitcoin
Core node over JSON-RPC, and builds, signs and broadcasts transactions. It also
decodes a broadcast transaction back from the node and verifies its signatures from
first principles, which is the one job here that does not go through BDK.

The assignment brief is preserved in `ASSIGNMENT.md`.

## Quick start

You need Rust stable and Bitcoin Core on your `PATH`. The demo was run against
v30.2.0.

```bash
cd rfb_labs_week_6

# A throwaway regtest node, wallet disabled, in ./.regtest
./scripts/regtest-node.sh start

# Configuration. Generate a seed and put it in .env.
cp .env.example .env
cargo run --quiet -- new-mnemonic
$EDITOR .env

# Then the whole walkthrough, from an empty chain to two verified transactions
./scripts/demo.sh
```

`.env` is gitignored and the seed never leaves it. The defaults in `.env.example`
match the node that `regtest-node.sh` starts, so the only value you have to fill in
is `RFB_MNEMONIC`.

To point at a node you already run, set `RFB_RPC_URL` and either `RFB_RPC_USER` and
`RFB_RPC_PASSWORD` or `RFB_RPC_COOKIE`. With none of those set the wallet looks for
the cookie file in the default Bitcoin Core data directory for the network. Setting
`RFB_NETWORK=mainnet` is refused.

## Commands

The binary is `rfbwallet`. Run it with `cargo run --` while developing, or build a
release binary once and call it directly.

| Command | What it does |
|---|---|
| `new-mnemonic` | Print a fresh 12 word BIP39 mnemonic. Touches nothing else. |
| `init --descriptor wpkh\|tr` | Create the database and derive both keychains. |
| `info` | Descriptors, keychain indices, checksums, chain state, storage. |
| `address [--change] [--unused] [--peek N]` | Reveal or inspect an address. |
| `balance` | Confirmed, pending, immature and spendable. |
| `utxos [--all]` | Unspent outputs with keychain, derivation index and maturity. |
| `txs` | Transaction history with net effect and fee. |
| `sync [--from-height N]` | Pull blocks and mempool from the node. |
| `send --to ADDR --amount SAT [--fee-rate N] [--selection ALGO] [--drain] [--dry-run]` | Build, sign and broadcast. |
| `verify TXID` | Decode a transaction and check its signatures. |
| `node` | What the node reports. |
| `mine --blocks N [--to ADDR]` | Mine regtest blocks, by default to this wallet. |
| `compare` | The `wpkh` and `tr` descriptors from the same seed, side by side. |

Every command exits non-zero with a readable message on failure. A few examples:

```text
$ rfbwallet send --to tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx --amount 1000
error: address `tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx` is not valid for regtest

$ rfbwallet send --to bcrt1p... --amount 99999999999999
error: insufficient funds: wallet holds 199.39999695 BTC spendable, transaction needs 1000000.00000786 BTC

$ RFB_MNEMONIC= rfbwallet send --to bcrt1q... --amount 1000
error: no mnemonic available: set RFB_MNEMONIC in your .env so the wallet can rebuild
its signing keys (the database only ever stores public descriptors)
```

## How it is put together

```text
src/
  main.rs        entry point, logging, error reporting, exit codes
  cli.rs         clap definitions
  commands.rs    one function per subcommand
  config.rs      environment and .env loading, network and RPC auth resolution
  error.rs       one error type, no panics on bad input
  keys.rs        seed handling and descriptor construction
  store.rs       wallet metadata table alongside BDK's own
  wallet.rs      create, open and persist the BDK wallet
  sync.rs        chain sync driven by the RPC block emitter
  node.rs        everything that talks to bitcoind
  tx.rs          building, signing and coin selection
  raw.rs         independent transaction verification, no BDK involved
  ui.rs          terminal output
scripts/
  regtest-node.sh  start, stop and reset a throwaway node
  demo.sh          the full walkthrough
evidence/
  demo-transcript.md  unedited output of a complete run
```

### Descriptor structure

The wallet is two descriptors derived from one account key. For the native SegWit
case they are:

```text
wpkh([43eb543c/84'/1'/0']tpubDCeP.../0/*)#3q86s9fl   external, receiving
wpkh([43eb543c/84'/1'/0']tpubDCeP.../1/*)#q5zmdse8   internal, change
```

The key origin at the front records the master fingerprint and the account path, so
the descriptor is enough on its own to say which seed and which account it belongs
to. The account key is hardened at three levels and the two keychain branches sit
unhardened below it. That split is what lets a single account xpub derive every
address on both branches, which is the whole basis of the watch-only mode described
below.

`keys.rs` assembles these strings by hand with `rust-bitcoin`'s BIP32 types rather
than using BDK's `Bip84` and `Bip86` templates. The templates produce the same
output. Writing it out keeps the origin, the account path and the branch split
visible in one place, which is the part of a descriptor wallet worth being explicit
about. It also kept `--descriptor tr` down to a purpose number and a fragment name
on one enum instead of a second code path.

Taproot uses the same shape with purpose 86 and a `tr()` fragment. `compare` prints
both from the same seed:

```text
descriptor  account path  output bytes  witness wu  address chars
wpkh        84'/1'/0'     31            107         44
tr          86'/1'/0'     43            66          64
```

The witness column is miniscript's own worst case satisfaction weight, not an
estimate. It shows the trade cleanly: taproot costs 12 more bytes in the output and
gives back 41 weight units on the spend.

### Where the private keys live

The SQLite file holds public data only. BDK's changeset stores
`Descriptor<DescriptorPublicKey>` for each keychain, never the secret key map, so
the database contains the public descriptors, the chain data and the transaction
graph and nothing that can spend. Private keys are rebuilt from `RFB_MNEMONIC` at
every open and passed to `LoadParams::extract_keys`.

Two things follow. Someone who takes the database file gets the wallet's history and
its addresses, but cannot move funds. And the wallet still opens without a seed, in
which case `info` reports `can sign  no` and any command that would sign fails with
an explanation instead of a panic. That is the watch-only mode, and it came out of
the storage design rather than being bolted on.

One extra table, `rfb_wallet_meta`, sits next to BDK's tables and records the script
type, the birthday height and the master fingerprint. BDK does not persist the script
type because it does not need to, but reopening does: the wallet has to know whether
to rebuild `wpkh` or `tr` descriptors from the seed before it can check them against
what was stored.

## Which library does what

`rust-bitcoin` owns the primitives. BIP32 derivation and the account xprv in
`keys.rs`, address parsing and network validation in `commands.rs`, and in `raw.rs`
consensus deserialisation, txid and weight computation, BIP143 and BIP341 sighashes
and secp256k1 signature verification. Anything that is a fact about Bitcoin rather
than a fact about this wallet is here.

`bitcoincore-rpc` is the only thing that touches the network. It is used for
`getblockchaininfo` to confirm the node is on the chain the wallet expects,
`getrawtransaction` for verification, `sendrawtransaction` to broadcast and
`generatetoaddress` to fund the regtest wallet. It is deliberately not used for any
wallet RPC. `regtest-node.sh` starts the node with `-disablewallet`, and the whole
demo runs against it, which is the proof: the node is a chain source and a relay,
and the wallet state lives entirely on this side.

`bdk_wallet` holds the state that is genuinely awkward to write yourself. Which
scripts belong to us across two keychains with a lookahead, which outputs are
unspent, what the balance is once coinbase maturity and unconfirmed change are
accounted for, and how to turn a spend request into a signed PSBT. Reimplementing
its script pubkey index and canonicalisation would have been the whole assignment
with none of it left over.

`bdk_bitcoind_rpc` joins the last two. Its `Emitter` walks the node forward block by
block from the wallet's stored checkpoint and hands each block back with the block it
connects to, so `apply_block_connected_to` can reject anything that does not line up.
That is the reorg handling, and it is the reason syncing from a bare node is not
just a `for` loop over `getblock`.

## Proof of a working transaction

Full transcript in `evidence/demo-transcript.md`. The short version, from a fresh
chain: the `wpkh` wallet mined 101 blocks to itself, then paid 1 BTC to the `tr`
wallet, which is a separate database from a separate seed.

```text
$ rfbwallet send --to bcrt1p4k2dlxmknrv6f3vvwfq4xeee908hnjcm5v36877hvhyuy44vdauqxkgh7l --amount 100000000 --fee-rate 2

transaction
  txid                  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  coin selection        branch-and-bound
  inputs                1
  outputs               2
  change                4899999695 sat (48.99999695 BTC)
  fee                   305 sat (0.00000305 BTC)
  effective fee rate    2.00 sat/vB (501 sat/kwu)
  weight                609 wu
  virtual size          153 vB

broadcast
  txid                  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  relayed to            http://127.0.0.1:18449
```

The node accepted it into its mempool, and after one block the receiving wallet found
it on its own by syncing from the same node:

```text
$ RFB_WALLET_DB=data/taproot.sqlite rfbwallet utxos

unspent outputs
  outpoint                                                            sat        keychain            index  status
  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de:0  100000000  external (receive)  0      confirmed @ 102

  count                 1
  total                 100000000 sat (1.00000000 BTC)
```

Then it sent 0.4 BTC back, spending a Taproot output with a key spend, and both
transactions verify from their raw bytes. Persistence is visible in the transcript
too: every command above is a separate process, and a second `sync` immediately
after the first applies zero blocks because the checkpoint is already at the tip.

## Stretch goals

All five are covered.

Multiple descriptor types: `init --descriptor wpkh|tr` picks the script type, it is
recorded in the metadata table so reopening rebuilds the right descriptors, and
`compare` puts both side by side with computed weights. The demo runs one wallet of
each and pays between them, so both signing paths and both verification paths are
exercised.

A CLI: `clap` with thirteen subcommands, described above. Nothing in the normal flow
requires editing code.

Explicit coin selection: `--selection bnb|largest-first|oldest-first`. BDK's
`coin_selection` changes the builder's type parameter, so `tx.rs` has one arm per
algorithm with the shared setup factored into a generic `configure`. The default
stays branch and bound because it is the one that can find a changeless solution.

Error handling and logging: one `Error` type in `error.rs` with a variant per
failure that a user can actually cause, `main` printing the cause chain with
duplicates suppressed, and `tracing` behind `--verbose` or `RFB_LOG`. There are no
`unwrap` calls on user input.

Raw `rust-bitcoin` over BDK: below.

## Where I reached for raw rust-bitcoin instead of BDK

BDK will tell you that a transaction it built is valid, but that is the wallet
marking its own homework. `verify TXID` answers a different question: is the thing
the node is holding actually the transaction we meant to send, and does its signature
really commit to the output it claims to spend. That is consensus plumbing, not
wallet state, and BDK has no API for it because it should not.

The command pulls the raw bytes back from `getrawtransaction`, deserialises them with
`bitcoin::consensus`, recomputes the txid from the decoded structure, resolves every
previous output, rebuilds the sighash for each input and verifies the witness
signature against the public key that the previous output's scriptPubKey commits to.
For the native SegWit case that is:

```rust
let sig = bitcoin::ecdsa::Signature::from_slice(&witness[0])?;
let pubkey = PublicKey::from_slice(&witness[1])?;

// The scriptPubKey commits to HASH160(pubkey); check the witness pubkey
// really is the one the output was locked to before checking the signature.
let expected = pubkey.wpubkey_hash()?;
if prev.script_pubkey.as_bytes()[2..] != expected.to_byte_array() {
    return Ok(false);
}

let sighash = cache.p2wpkh_signature_hash(
    index,
    &prev.script_pubkey,
    prev.value,
    sig.sighash_type,
)?;

let message = Message::from_digest(sighash.to_byte_array());
Ok(secp.verify_ecdsa(&message, &sig.signature, &pubkey.inner).is_ok())
```

The Taproot path in `raw.rs` does the same with `taproot_key_spend_signature_hash`
over all prevouts and `verify_schnorr` against the x-only key taken straight out of
the witness program. Output of both, against the two transactions from the demo:

```text
inputs
  #  spends                                                              sat         type                          signature
  0  9d2db688b726c5078dc31aef35d4a817136c93ae0456e3310d325d36df29be1b:0  5000000000  p2wpkh (segwit v0 key spend)  valid

inputs
  #  spends                                                              sat        type                      signature
  0  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de:0  100000000  p2tr (taproot key spend)  valid
```

`verify` falls back to the local transaction graph when the node cannot supply the
bytes, and says which source it used, because verifying against the node's copy is a
stronger statement than verifying against our own.

## Tests

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Twenty six tests, all passing, none ignored. The derivation tests check the
descriptors against the published BIP84 and BIP86 vectors for the standard
`abandon ... about` mnemonic, including the BIP86 account xpub itself, so a mistake
in the account path or the branch split fails the build rather than producing a
wallet that quietly derives the wrong addresses.

The verification tests in `raw.rs` carry the two transactions from
`evidence/demo-transcript.md` as raw hex, so the sighash and signature checks run
without a node. Two of them flip a byte inside the witness signature and assert the
result turns invalid, which is the test that the verifier is actually verifying and
not just returning true.

## Known limitations

Signature verification covers `p2wpkh` and `p2tr` key spends. Script path spends,
`p2wsh` and legacy inputs are reported as not checked rather than being guessed at.
Resolving previous outputs needs either `-txindex` or the inputs to be known to this
wallet, so `verify` on an unrelated confirmed transaction against a pruned node
reports the fee as unknown and skips the signature check instead of failing.

Sync is a full pass over the emitter with no compact block filters, which is fine on
regtest and would be slow on testnet from height zero. `--from-height` and the
recorded birthday are the mitigation, not a solution. There is no rescan command, so
importing an existing seed into a fresh database means syncing from the birthday you
give it.

Coin selection is BDK's, with the algorithm exposed. There is no manual UTXO
selection or freezing, although `TxBuilder::add_utxos` and the locked outpoints API
are both there to build on. Fee estimation uses whatever `estimatesmartfee` returns
and falls back to the `--fee-rate` value, which on regtest is always the fallback.

RBF and fee bumping are not wired up even though `build_fee_bump` exists, and there
is no PSBT import, so the wallet cannot act as one signer among several. Both would
be the next things I added.

The `mine` command exists to make regtest usable and is refused on any other network.

## References

- [rust-bitcoin](https://docs.rs/bitcoin/0.32.102/bitcoin/index.html)
- [bdk_wallet](https://docs.rs/bdk_wallet/3.1.0/bdk_wallet/index.html)
- [bdk_bitcoind_rpc](https://docs.rs/bdk_bitcoind_rpc/0.22.0/bdk_bitcoind_rpc/index.html)
- [rust-bitcoincore-rpc](https://docs.rs/bitcoincore-rpc/0.19.0/bitcoincore_rpc/index.html)
- [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki),
  [BIP39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki),
  [BIP84](https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki),
  [BIP86](https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki)
- [BIP143](https://github.com/bitcoin/bips/blob/master/bip-0143.mediawiki) and
  [BIP341](https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki) for the
  two sighash algorithms in `raw.rs`
