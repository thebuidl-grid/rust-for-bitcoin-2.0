# Week 6, explained from scratch

A plain-language walkthrough of what the wallet actually does under the hood, using
the real values from the end-to-end run done while building it — mnemonic,
descriptors, txid, balances, all real output from this code against a live regtest
node, not made up for illustration.

Weeks 1-5 built understanding one layer at a time: run a node, model a transaction as
data, parse real transactions, derive keys and addresses by hand. Week 6 assembles
those layers into something that behaves like an actual wallet — it remembers state
between runs, watches a live chain, and can spend money.

---

## Part 1 — What each library is actually for

Three crates, three different jobs, and it matters which one owns which:

**`bdk_wallet`** is the wallet itself. Feed it two descriptors (external and internal,
see Part 2) and it handles everything that makes "a wallet" different from "a key":
tracking which addresses have been revealed, which UTXOs belong to it, computing
balance, building and signing PSBTs, and persisting all of that to SQLite. This is the
layer that would be enormous and error-prone to hand-roll — coin selection alone is a
whole subfield.

**`bdk_bitcoind_rpc`** is the bridge between that wallet state and a live node. Its
`Emitter` type walks blocks forward from wherever the wallet last left off and hands
them to `bdk_wallet` in a form it can apply directly. This is the actual mechanism
behind "the wallet is not purely offline" — every `sync` call in this project makes
real RPC calls to a running `bitcoind`.

**`bitcoincore_rpc`** (re-exported through `bdk_bitcoind_rpc`, so it never needed its
own line in Cargo.toml) is the raw RPC client. `bdk_wallet` doesn't broadcast
transactions or mine blocks for you — those are just one-shot RPC calls, so this
project calls `send_raw_transaction` and `generate_to_address` on it directly instead
of routing them through BDK.

---

## Part 2 — One mnemonic, two descriptors, two keychains

Everything below came out of a real `init` run:

```
mnemonic:  save bean setup anger depth final moment oyster ability soda dwarf term

external descriptor:
wpkh(tprv8ZgxMBicQKsPe7C9S4Zv6dM6MUfXjZMS59JfuSzhUF6kcoZtqwnKxLjRPdHP25BaEZs2Un5TRMddJ4cNwuAWE464yNSjReTUgvJiSA82HER/84'/1'/0'/0/*)#27stngsp

internal descriptor:
wpkh(tprv8ZgxMBicQKsPe7C9S4Zv6dM6MUfXjZMS59JfuSzhUF6kcoZtqwnKxLjRPdHP25BaEZs2Un5TRMddJ4cNwuAWE464yNSjReTUgvJiSA82HER/84'/1'/0'/1/*)#m242waqe
```

That's the same tree-of-keys idea from Week 5, wired into a real wallet. One master
key (from the mnemonic's seed) derives two branches — `.../0/*` and `.../1/*` — the
exact receive/change split from BIP44. `bdk_wallet` calls these `KeychainKind::External`
and `KeychainKind::Internal`. `wpkh(...)` at the front says every address either
branch produces is native SegWit (BIP84). The `#27stngsp` / `#m242waqe` suffixes are
descriptor checksums — same idea as an address checksum, catching a typo in the
descriptor string itself before it gets used.

`cargo run -- address` calls `reveal_next_address(KeychainKind::External)`, which
advances the external branch by one index and gives back a fresh address —
`bcrt1qgrqm9y2a62eckqn58weaklzdd60tnr4rehn7mh` was the very first one revealed in this
run. `change-address` does the same thing on the internal branch. Each reveal is
persisted immediately, so the wallet never hands out the same address twice even
across restarts.

---

## Part 3 — Sync: turning "on disk" into "actually current"

```rust
let wallet_tip = wallet.latest_checkpoint();
let mut emitter = Emitter::new(client, wallet_tip, 0, unconfirmed_txs);
while let Some(block_emission) = emitter.next_block()? {
    wallet.apply_block_connected_to(&block_emission.block, height, connected_to)?;
}
let mempool_event = emitter.mempool()?;
wallet.apply_evicted_txs(mempool_event.evicted);
wallet.apply_unconfirmed_txs(mempool_event.update);
wallet.persist(db)?;
```

`latest_checkpoint()` is "where did we leave off" — the wallet's own record of the
last block it processed. `Emitter::new` starts from there, not from genesis, so a
second `sync` call on an already-synced wallet does almost no work: it asks the node
"anything past this checkpoint?" and gets back nothing new to apply. That's the actual
end-to-end proof this project ran: mine 101 blocks, `sync`, see `confirmed: 100 BTC`
appear and `immature: 4950 BTC` sitting behind it (100 blocks' worth of coinbase
outputs still short of their 100-confirmation maturity). One more mined block later,
one of those matures.

Applying a block isn't the whole story — `mempool()` also asks the node about
unconfirmed transactions and evictions, which is how a broadcast transaction shows up
in the wallet's view before it's mined. Every `sync`, `utxos`, and `send` in this CLI
does this whole sequence before doing anything else, specifically so the wallet is
never acting on stale local state.

---

## Part 4 — Build, sign, broadcast: a PSBT in three calls

```rust
let mut builder = wallet.build_tx();
builder.add_recipient(destination.script_pubkey(), Amount::from_sat(amount_sats))
       .fee_rate(feerate);
let mut psbt = builder.finish()?;

let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
let tx = psbt.extract_tx()?;
let txid = client.send_raw_transaction(&tx)?;
```

`build_tx()` runs coin selection (choosing which UTXOs to spend), calculates change,
and hands back an unsigned PSBT — a standardized format for a transaction that isn't
signed yet, carrying enough metadata (which UTXOs, which descriptor paths) that a
signer doesn't need any outside context to know what it's authorizing. `wallet.sign`
walks every input, finds the private key that matches its derivation path (the wallet
holds the descriptor's key material because it's the one used in Part 2 — a
watch-only setup with only public descriptors would fail here, same idea as the
xpub-can't-derive-hardened-children limit from Week 5), and produces the witness data.
`extract_tx()` turns the now-fully-signed PSBT into an actual `Transaction` ready to
broadcast.

From the real run:

```
sending 1,000,000 sats
Broadcast txid: 8240178d409ffa3580c92c6d929bc186263eb95556e0d7b55a7dd3345d741c30
```

`bitcoin-cli getrawtransaction <txid> true` confirmed it sitting in the node's
mempool immediately after — real broadcast, real node accepting it, not a local-only
simulation. One more mined block later, `utxos` showed three new entries from that
transaction: the 0.01 BTC payment, the change output back to the internal keychain,
and (since this was a self-send) the payment landing back in the external keychain
too.

---

## Part 5 — Why this survives a restart

Every command in this CLI is a fresh `cargo run` — a brand new process, no shared
memory with the previous command. The only thing carrying state between them is
`wallet.sqlite`. `Wallet::load()` at the top of every command reopens that file,
checks the descriptors passed in still match what's stored, and reconstructs the
exact wallet state — every revealed address, every known UTXO, every checkpoint —
without re-deriving or re-scanning anything from scratch. That's the practical
difference between "a script that computes an address" (Week 5) and "a wallet" (this
week): state that outlives the process.

Proof from the run: `sync` was called in one process, showing balance `100 BTC`
confirmed. A completely separate `cargo run -- send ...` process afterward, then a
third `cargo run -- sync` process, both picked up exactly where the previous one left
off — same UTXO set, same revealed-address index, no re-sync-from-genesis needed.

---

## What to check against your own run

```bash
cargo run -- init                 # mnemonic + descriptors above
cargo run -- mine 101 && cargo run -- sync    # 100 BTC confirmed, ~4950 BTC immature
cargo run -- address              # a fresh bcrt1q... address
cargo run -- send <address> 1000000
cargo run -- mine 1 && cargo run -- utxos     # the new outputs from that send
```
