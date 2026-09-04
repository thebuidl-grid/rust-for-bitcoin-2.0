# Lab 05 — Address compatibility map

## Commands used

```
cargo test --test lab_05 -- --nocapture
```

Ad-hoc check of a P2SH-era wallet's capabilities (Base58Check P2PKH and P2SH support,
no Bech32/Bech32m):

```rust
let legacy_wallet = SenderCapabilities { base58_p2pkh: true, base58_p2sh: true, bech32: false, bech32m: false };
lab05_compatibility::compatibility_report(legacy_wallet)
lab05_compatibility::best_supported_format(legacy_wallet)
```

## Terminal output

```
running 4 tests
test builds_the_four_format_map ... ok
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
compatibility_report = CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }
best_supported_format = Some(P2sh)
```

Enabling `bech32` flips `best_supported_format` to `P2wpkh`; additionally enabling
`bech32m` flips it to `P2tr`, matching `selects_the_most_modern_supported_format`.

## Evidence references

- `cargo test --test lab_05` output above.
- Source: `src/labs/lab05_compatibility.rs`.
- Test suite: `tests/lab_05.rs`.

## Explanation

A `3...` address is still Base58Check-encoded, exactly like the `1...` P2PKH addresses
that predate SegWit, so any wallet old enough to send Bitcoin at all already has the
Base58Check decoder needed to pay it — it just sees "pay to this script hash" and has
no idea a witness program is wrapped inside. A `bc1q...` address, however, uses an
entirely different text encoding, Bech32, introduced by BIP173 alongside SegWit. A
wallet whose sending code was written before BIP173 has no Bech32 decoder at all, so
`bc1q...` is not an unsupported *script type* to it, it is unparsable *text* — the
wallet can't even recognize it as an address. This is also why receiving support and
sending support are different questions: a modern wallet can *receive* at a
`bc1q...`/`bc1p...` address it generated for itself, but whether someone paying it can
*send* to that address depends entirely on whether the sender's own software has been
updated to understand Bech32/Bech32m, which is exactly what `SenderCapabilities`
models.
