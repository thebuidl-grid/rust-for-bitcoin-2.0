# Lab 05 — Address compatibility map

## Commands used

```
cargo test --test lab_05 -- --nocapture
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

## Evidence references

For a P2SH-era wallet (`base58_p2pkh: true, base58_p2sh: true, bech32: false,
bech32m: false`):

```
compatibility_report: CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }
best_supported_format: Some(P2sh)
```

Enabling `bech32` shifts `best_supported_format` to `P2wpkh`; enabling `bech32m` on top of
that shifts it to `P2tr` — confirmed by `selects_the_most_modern_supported_format`.
`required_encoding` maps P2PKH/P2SH to `"Base58Check"`, P2WPKH to `"Bech32"`, and P2TR to
`"Bech32m"`.

## Explanation

An older, P2SH-era wallet can still accept a `3...` address because Base58Check-encoded
P2SH addresses were introduced alongside P2PKH and have been part of the wallet-sending
code path since well before native SegWit existed — the wallet only needs to know how to
Base58Check-decode a version byte and build a standard `OP_HASH160 ... OP_EQUAL` output.
It rejects `bc1q...` because Bech32 is a completely different text encoding introduced by
BIP173 for native SegWit outputs; a wallet that was never updated to parse Bech32 has no
code path that recognizes the `bc1` human-readable part or its checksum, so the address
is simply illegible to it, not merely "unsupported by policy." This is why *send*
support and *receive*/*spend* support are different questions: a wallet can safely spend
funds it received on any address type it can construct a valid unlocking script for, but
it can only *send to* addresses whose encoding and script template it also knows how to
build client-side — and that sending capability lags behind script-level and consensus
support because it requires a software update on the sender's end specifically, not just
the network's.
