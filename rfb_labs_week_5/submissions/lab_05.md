# Lab 05 — Address compatibility map

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_05
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test builds_the_four_format_map ... ok
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

For a "P2SH-era" wallet (`base58_p2pkh: true, base58_p2sh: true, bech32: false, bech32m: false`):

```
P2SH-era wallet report: CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }
can send to bc1q...:    false
can send to 3...:       true
```

## Evidence references

- `src/labs/lab05_compatibility.rs` — `can_send_to`, `compatibility_report`,
  `best_supported_format`, `required_encoding`.
- `tests/lab_05.rs::selects_the_most_modern_supported_format` — as `bech32`/`bech32m`
  get switched on one at a time, `best_supported_format` moves
  P2sh → P2wpkh → P2tr, matching the priority order Taproot > native SegWit >
  wrapped SegWit > legacy.

## Explanation

This lab is really about the difference between a wallet being able to *understand* an
address string and a wallet being able to *spend* what that address locks. Sending
support is what `can_send_to`/`compatibility_report` model: can the sender's software
parse the address and build an output with the right scriptPubKey for it? Spending
support is a different question entirely — can whoever *receives* the funds actually
unlock that output, and does the network as a whole enforce the rules that make the
new script type valid? An old wallet has zero say over whether the network accepts
SegWit transactions; that's a consensus-level fact, not a wallet feature.

A P2SH-era wallet — one written after BIP16 but before bech32/BIP173 shipped — can
send to `3...` addresses without any trouble, because Base58Check decoding and
`OP_HASH160`/`OP_EQUAL` scriptPubKey construction were already old news by then. It
can *also* send to a P2SH-wrapped SegWit address like `3J...`, without knowing
anything about SegWit at all, because as far as that wallet's concerned it's just
building an ordinary P2SH output — the fact that the redeemScript inside happens to be
a witness program only matters once someone tries to spend it. That's the whole trick
behind BIP49: wrap the new thing in the old thing's clothing so old software doesn't
need to change.

`bc1q...` is a different story. Bech32 isn't Base58Check with a different prefix,
it's a genuinely different string encoding (a different character set and a different
checksum algorithm), defined by BIP173 well after the P2SH era. A wallet that's never
had bech32 decoding added to it doesn't fail to send to `bc1q...` because it disagrees
with SegWit philosophically — it fails because it literally cannot parse the string.
The `1` separator, the character set, the checksum: none of that means anything to
software that only knows Base58Check. So the same wallet happily accepts `3...`
(old encoding, and it doesn't even need to know what's behind the hash) but rejects
`bc1q...` (an encoding it was never taught to read), even though under the hood both
addresses might ultimately protect a witness program.
