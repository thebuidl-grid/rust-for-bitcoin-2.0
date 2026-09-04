# Lab 05 — Sender compatibility map

## Commands used

```bash
cargo test --test lab_05 -- --nocapture
```

## Terminal output

```
running 4 tests
test names_the_required_human_encoding ... ok
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_05.rs` pass against `src/labs/lab05_compatibility.rs`: a
P2SH-era wallet (`base58_p2pkh`/`base58_p2sh` only) can send to `P2sh` but not `P2wpkh`;
`compatibility_report` builds the correct four-format map from that wallet's capabilities;
`best_supported_format` upgrades its answer from `P2sh` → `P2wpkh` → `P2tr` as `bech32`/`bech32m`
support is added; `required_encoding` returns `Base58Check`/`Base58Check`/`Bech32`/`Bech32m` for
`P2pkh`/`P2sh`/`P2wpkh`/`P2tr` respectively.

## Explanation

An older wallet that only understands Base58Check (P2PKH `1...` and P2SH `3...`) accepts `3...`
addresses because, to that wallet, a P2SH address looks like any other Base58Check-encoded
20-byte hash with a specific version byte — there's nothing in the *encoding* that distinguishes
"P2SH wrapping a SegWit witness program" from "P2SH wrapping a multisig script." The wallet just
builds a standard `OP_HASH160 <hash> OP_EQUAL` output; it never needs to understand what's inside
the redeemScript to construct that payment. `bc1q...` addresses, by contrast, use an entirely
different encoding (Bech32) that a Base58-only wallet's parser was never written to recognize at
all — it's not that the wallet disapproves of native SegWit, it's that the string doesn't parse as
a valid address in its model, so it's rejected before any semantic question about the output type
even comes up.

This is exactly why **sending support and spending support are different questions**. Sending
support depends only on whether your wallet's *address parser and output-construction code*
understands the destination format — which is a property of the sender's own software age/design,
independent of the coins involved. Spending support depends on whether *the transaction's own
inputs* are constructed in a way nodes on the network will validate (which script versions/opcodes
are active consensus rules). A wallet can very well be able to spend from a P2WPKH input it already
owns (because validating an existing UTXO uses different code paths / because the network enforces
SegWit rules regardless of wallet age) while still being unable to *construct a new* P2WPKH output
because its own address book / UI never learned to parse or offer `bc1q...` as a destination.
`best_supported_format`'s escalation (P2SH → P2WPKH → P2TR as capabilities are added) models this
sending-side gap directly: it's about what the wallet can *address*, not what it can eventually
*validate on-chain*.
